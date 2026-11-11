# 0006. aarch64 under QEMU, behind x86_64

- **Status:** Accepted
- **Date:** 2026-11-11

## Context

design.md listed aarch64 bring-up as a non-goal and left open which loader would boot it, since `bootloader` supports only x86_64. Between milestone 3's second and third work items, a spike on its own branch tried the generic kernel on aarch64 under QEMU. It reached milestone 2's parity in seven steps: boot, frames, page tables, traps, heap, timer and a test exit, with all 31 kernel tests passing. Only the arch crate and a device tree reader are new; the kernel changed in two generic places, `Trap::Stray` widening to `u32` and kernel stacks aligned to a power of two at or above twice their size.

Keeping the code on a branch would leave it to conflict with every later change to the kernel and the HAL. Porting each milestone 3 item to both architectures as it lands would roughly double the architecture work in a milestone whose real target is the x86_64 laptop.

## Decision

aarch64 is a second architecture, on QEMU's virt machine only: `cortex-a76`, GICv3, TCG, ramfb and edk2 from the same `ovmf-prebuilt` release as x86_64. Limine 12.9.6 boots it, fetched by xtask at that release and checked against a SHA-256, while x86_64 stays on `bootloader`. QEMU runs with `acpi=off`, because edk2 otherwise gives Limine ACPI tables and no device tree, and the kernel finds the GIC and the timer in the device tree through `lib/fdt`. `thisos-arch-aarch64` joins `thisos-arch-x86_64` as a crate allowed unsafe code.

aarch64 runs behind x86_64, with no deadline to catch up:

- Work items are built and tested for x86_64 first. A HAL call that aarch64 doesn't implement yet panics there with `<call> isn't ported to aarch64`.
- Every work item keeps aarch64 building and lint-clean, since `cargo xtask ci` builds and lints it.
- aarch64 gains a feature when a decision round schedules the port.
- A milestone's closing notes list what aarch64 still lacks.

## Consequences

Each new HAL call costs a stub in the aarch64 crate, and a change to generic code has to build for both targets. There are two loaders to keep up with. The aarch64 QEMU tests run only through `cargo xtask test --arch aarch64`: neither `cargo xtask ci` nor GitHub CI runs them, and the runner image has no `qemu-system-aarch64`, so a regression there shows up only when someone runs them.

aarch64 has no real hardware target, no ACPI support and, for now, no serial console: the kernel logs to the ramfb framebuffer alone. Anything past milestone 2, starting with threads, lands on x86_64 only until its port is scheduled.
