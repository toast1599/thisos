# ThisOS design

This document describes what ThisOS is and how its parts fit together. It covers what is decided and what the next milestone needs, not the whole system in advance. How code is written, tested and reviewed is in [CONTRIBUTING.md](../CONTRIBUTING.md). Decisions that are hard to reverse have a short record in [adr/](adr/).

## Goals

ThisOS is a general-purpose operating system written in Rust, built on a microkernel. Safety means three things here:

- The kernel is memory-safe Rust. Its unsafe code sits in one crate per architecture, documented and tested.
- Programs are isolated by the hardware: each runs in its own address space in user mode.
- No program has ambient authority. A program can reach only the kernel objects it holds handles to, and a new program starts with only the handles its creator gives it.

ThisOS is not Unix-like and has no libc. Programs target Rust's standard library once it is ported (milestone 7); until then they use `no_std`, `alloc` and the ThisOS runtime crate.

## Non-goals for now

These are deferred. Each comes back through a decision round when a milestone needs it.

- More than one CPU core (SMP)
- aarch64 and riscv64 bring-up
- The IOMMU
- Networking and USB
- Image signing, Secure Boot support, updates and packages
- Resource quotas and accounting
- Hardening against speculative-execution side channels beyond what the CPU and firmware do by default
- An interface definition language for IPC
- Vendoring dependencies or recording their publishers

## Architectures and hardware

ThisOS is 64-bit only. Architecture names follow Rust's `target_arch`.

- **x86_64** is the only architecture built and tested for now. It runs in QEMU and on real hardware, and both must pass the same tests.
- **aarch64** and **riscv64** are the long-term targets. They will be tested in QEMU only, which is a documented limitation. No code for them exists yet.

QEMU runs the q35 machine with OVMF firmware and emulates the same devices as the test laptop: an NVMe disk and an i8042 keyboard, with no virtio. One driver per device class then serves both.

The real-hardware target is a Lenovo 83DA laptop:

- Core Ultra 5 125H (Meteor Lake, 18 threads, mixed P- and E-cores)
- UEFI only
- NVMe SSD and an i8042 keyboard
- VT-d with interrupt remapping
- Wi-Fi only, no serial port

Unsigned images need Secure Boot turned off in its firmware settings.

## Boot

The kernel boots through UEFI only, using the `bootloader` crate (0.11) unforked. The kernel is an ELF built for `x86_64-unknown-none`. `bootloader` maps all of physical memory at an offset it picks, and the kernel reaches any frame through that mapping. Its ramdisk carries `init` and the other early programs.

`bootloader` supports only x86_64. The arch crate converts its `BootInfo` into the kernel's own boot structure in one place, so a different loader for another architecture changes one file. A fork of `bootloader` for aarch64 is under consideration.

## Crates

```
ThisOS/
  Cargo.toml          workspace for target crates
  .cargo/config.toml  default target x86_64-unknown-none
  kernel/             thisos-kernel: the kernel binary, generic
  hal/                thisos-hal: traits the kernel needs from an architecture
  arch/x86_64/        thisos-arch-x86_64: implements thisos-hal
  abi/                thisos-abi: syscall numbers, errors, handle and rights types
  lib/*/              host-testable no_std logic
  user/rt/            thisos-rt: user runtime (entry, syscalls, allocator, panic)
  user/init/          the first process
  xtask/              a separate workspace: image building, QEMU, tests, font
```

Dependencies point from specific crates to generic ones. `thisos-kernel` depends on `thisos-hal`, and, under `cfg(target_arch = "x86_64")`, on `thisos-arch-x86_64`, which also depends on `thisos-hal`. The kernel reaches the hardware only through the `thisos-hal` traits and never names an architecture's types. `thisos-arch-x86_64` is the only crate that may depend on the `x86_64` crate, so architecture code leaking into generic crates shows up as a dependency change.

All crates are named with the `thisos-` prefix. A new crate is added only when its dependencies, unsafe status or test environment differ from its neighbors'; otherwise the code is a module.

## Kernel

The kernel owns these mechanisms ([0001](adr/0001-kernel-scope.md)):

- physical memory, page tables and a kernel heap
- threads and a preemptive scheduler
- handles and the kernel objects they name
- IPC channels and shared memory
- delivery of interrupts to user space

Drivers, filesystems and networking run in user space. Something moves out of the kernel only when there is evidence for it.

The kernel logs through the `log` facade to two places: the serial port (COM1), which QEMU tests read, and a framebuffer text console, which is all the laptop has. The console uses JetBrains Mono rasterized to printable ASCII at 12, 16, 24 and 32 pixels per em, which give cells of 7×16, 10×22, 14×32 and 19×43 pixels. It uses the largest size that gives at least 40 rows, or the smallest when none does: 12 px on QEMU's 1280×800 screen (50 rows), 16 px on 1080p and on the test laptop's 1920×1200 (49 and 54 rows), and 32 px at 4K (50 rows). It keeps the text in a grid of cells and, when it scrolls, redraws from that only the cells whose character or color changes, because the firmware maps the framebuffer uncached: reading it is slow, and every store is a bus transaction. It writes two pixels per 8-byte store. On the test laptop, redrawing the whole screen with one store per byte took 1.34 s. On x86_64 the arch crate also maps the framebuffer write-combining, so that the CPU gathers its stores into bursts: it changes PAT entry 1 from WT to WC and selects it on the framebuffer's pages. It does so only when the MTRRs make the whole framebuffer UC, because the loader's mapping of all physical memory is a second mapping of it, and a WC page must not also be mapped cacheable. `BootInfo` says whether it did, and the kernel logs it.

A kernel panic prints its message, its location and a backtrace that follows frame pointers to both log sinks, then halts every core. The kernel is built with frame pointers for this. The backtrace stays inside the kernel stack and stops after 32 frames. Its addresses are link addresses, one byte into each call, so `addr2line -e` on the kernel ELF names them directly; the kernel holds no symbols. `core` comes prebuilt without forced frame pointers, so a frame inside it can be missing or end the walk early. A panic that finds the logger locked came from inside the logger, so it writes to a second handle on the serial port only and skips the console. A panic during the report halts at once. In a test build a panic ends the test run on a summary of the results, then exits QEMU with a failure code, or halts on real hardware.

## Handles

Kernel objects are reached through typed handles ([0002](adr/0002-handles.md)). Each handle names one object of a fixed type, such as a channel, memory object, thread, process or interrupt.

A handle carries rights from a fixed set: read, write, execute (memory objects only), duplicate and transfer. Rights can be removed and never added. Sending a handle over a channel moves it out of the sender. The kernel has no global names: a process reaches an object only through a handle it was given. In user-space Rust, a handle is a non-`Copy` type such as `Handle<Channel>` that closes on drop.

## IPC

Processes talk through kernel-buffered, bounded, two-way channels that carry bytes and handles. A thread can wait on several objects at once. Bulk data goes through shared memory objects. This is provisional until the milestone 4 prototype confirms it ([0003](adr/0003-ipc-channels.md)).

## Processes

The kernel parses one ELF file, `init`'s, from the ramdisk. Every other process is built by a loader in user space through general syscalls: create an address space, map memory into it, create a thread, start it ([0004](adr/0004-process-creation.md)).

## User-space drivers

Drivers and services contain no unsafe code. MMIO, port I/O and DMA buffers go through one audited crate with safe, typed APIs. Until the IOMMU is used, a driver that can program a device's DMA can still reach any physical memory, so drivers with DMA are trusted that far.

## Milestones

Each milestone ends with QEMU and the laptop doing something they couldn't before.

1. **Boot.** The workspaces and xtask; the kernel boots under OVMF and on the laptop; serial and framebuffer logging; the panic handler; QEMU tests; CI passing.
2. **Kernel core.** GDT, TSS and IDT; exceptions with trap frames; a frame allocator built from the memory map; page tables owned by the kernel; the heap; the LAPIC timer.
3. **User mode.** Address spaces, threads, context switching, preemptive round-robin scheduling and syscall entry. `init` loads from the ramdisk, prints through a syscall and exits.
4. **Handles and IPC.** The per-process handle table, channels that carry handles, shared memory and a wait call. `init` starts two processes that talk to each other.
5. **User-space drivers.** Interrupts delivered to user space; handles for MMIO and I/O port ranges; serial and i8042 keyboard drivers in user space; a shell that runs programs from the ramdisk.
6. **Storage.** PCI enumeration in user space, an NVMe driver, a filesystem server and a namespace protocol.
7. **std.** A std platform-layer patch over the ABI as it stands then, giving `std::{io, fs, env, process, thread, time}`. An earlier spike showed this route works.

## Open questions

- How a handle is represented (for example, table index plus generation), and the syscall calling convention. Due at milestones 3 and 4.
- The heap allocator crate and the frame allocator's algorithm. Due at milestone 2.
- How the kernel reads and writes its page tables. The `x86_64` crate reaches a table through `&mut PageTable`, but the CPU's page walker sets Accessed and Dirty bits in that memory while the reference lives, which Rust's aliasing rules don't allow, and with more than one core another core's walker does too. The PAT setup at boot has the same flaw and gets away with it only because it runs once, on one core, and nothing reads those bits. Due at milestone 2, when the kernel owns its page tables.
- How the kernel logs from an interrupt or exception handler. The logger's lock is a spin lock, so a handler that logs while the code it interrupted holds the lock never returns. Due at milestone 2.
- How a panic inside the framebuffer console reaches the screen. Its report goes to serial only, which the laptop doesn't have. Due at milestone 2, when the shared test support module lets a test panic inside the logger.
- The name and API of the audited driver crate. Due at milestone 5.
- Networking on the laptop: its only network device is Intel CNVi Wi-Fi, so matching QEMU there would need a Wi-Fi driver or a USB Ethernet adapter, which needs xHCI first.
- Which loader boots aarch64 and riscv64.
