# 01. Objective, scope and trust

- **Day:** 1 (Wed 2026-10-14)
- **Status:** Frozen

## Covers

What the OS is for and how much it takes on, the threat model it defends against, and what the system trusts.

## Decided

- [0001](../decisions/0001-general-purpose-os.md) General-purpose production OS
- [0002](../decisions/0002-kernel-mode-verified-execution.md) All code runs in kernel mode, isolated by verification (superseded by [0014](../decisions/0014-microkernel.md))
- [0003](../decisions/0003-rust-std-no-libc.md) Rust is the system language; the Rust standard library replaces libc (amended by [0082](../decisions/0082-corrections.md) and [0221](../decisions/0221-corrections.md))
- [0004](../decisions/0004-not-unix.md) Not Unix-like: `target_family` is not `unix`
- [0008](../decisions/0008-architecture-neutral-design.md) Architecture-neutral design (amended by [0082](../decisions/0082-corrections.md))
- [0009](../decisions/0009-x86-64-first.md) x86_64 is the first architecture
- [0010](../decisions/0010-64-bit-only.md) 64-bit architectures only
- [0011](../decisions/0011-aarch64-second.md) AArch64 is the second architecture
- [0014](../decisions/0014-microkernel.md) Microkernel: a minimal, audited kernel in kernel mode; everything else in user mode (amended by [0082](../decisions/0082-corrections.md) and [0169](../decisions/0169-iommu-service.md))
- [0015](../decisions/0015-clean-apis.md) APIs and ABIs are designed for this system: inspired by others, never bound to them
- [0016](../decisions/0016-threat-model-scope.md) Threat model: speculative leaks between programs and denial of service are in scope (amended by [0202](../decisions/0202-offline-attackers.md))
- [0019](../decisions/0019-kernel-minimality.md) Kernel minimality: anything that can reasonably run in user mode does
- [0045](../decisions/0045-aarch64-with-first-user-program.md) AArch64 is brought up together with the first user-mode program (amended by [0254](../decisions/0254-corrections.md))
- [0046](../decisions/0046-scope-of-0008.md) Scope of 0008: the OS's own code, including platform-specific code (amended by [0152](../decisions/0152-kernel-log-and-reset.md) and [0213](../decisions/0213-power-capability.md))
- [0116](../decisions/0116-name-thisos.md) The OS is named thisOS

The final review ([07-engineering](07-engineering.md)) added these on the threat model and on trust:

- [0202](../decisions/0202-offline-attackers.md) Attackers who get the machine while it's off are in scope
- [0214](../decisions/0214-interrupt-remapping.md) Devices raise only the interrupts they were given, through interrupt remapping
- [0215](../decisions/0215-pci-bus-driver-config.md) The PCI bus driver keeps configuration space, and mediates drivers' access to it
- [0216](../decisions/0216-essential-service-failure.md) A failure of an essential service is handled like a fault in the root task

How the kernel stays small was settled on Day 8 ([08-roadmap](08-roadmap.md)):

- [0247](../decisions/0247-kernel-size-ratchet.md) A record rule and a size ratchet keep the kernel small

## The kernel's contents

The rule comes from [0019](../decisions/0019-kernel-minimality.md): the kernel keeps only what can't reasonably run in user mode. We judged each mechanism on its own, starting from seL4's set (address spaces, threads and scheduling, IPC, interrupt delivery and capabilities), and the final review ([07-engineering](07-engineering.md)) closed the list. Adding anything to it after the freeze takes a new record ([0201](../decisions/0201-changes-after-freeze.md)). On top of that, a size ratchet keeps the kernel from growing without anyone noticing ([0247](../decisions/0247-kernel-size-ratchet.md)).

A good deal moved out to user mode: physical memory management [0025](../decisions/0025-user-mode-memory.md), page-fault handling [0027](../decisions/0027-user-mode-pagers.md), building page tables [0052](../decisions/0052-user-mode-page-tables.md), program loading [0031](../decisions/0031-user-mode-loader.md), most hardware discovery [0032](../decisions/0032-minimal-hardware-discovery.md) and scheduling policy [0088](../decisions/0088-user-mode-scheduler.md).

As for the kernel itself, it has no heap [0068](../decisions/0068-no-kernel-heap.md) and one stack per core [0060](../decisions/0060-kernel-stack-per-core.md), and the build checks how deep that stack goes [0198](../decisions/0198-stack-depth-checker.md). It never reads program data [0072](../decisions/0072-registers-only.md). Between interruption points it runs with interrupts off [0089](../decisions/0089-interrupts-off-in-kernel.md), and those points are also where it lets other cores take its one lock [0218](../decisions/0218-lock-yield.md). Its guards against touching or running program memory are always on [0075](../decisions/0075-user-guards-always-on.md), and it flushes the object window only in batches [0217](../decisions/0217-window-flush-batched.md).

On the hardware side, the kernel reads the interrupt controller and the timer from the firmware tables [0151](../decisions/0151-boot-info-from-crate.md). Its log goes to a serial port, and when it fails it draws the log on the framebuffer [0225](../decisions/0225-framebuffer-log-at-failures.md). It resets or powers off the machine itself ([0152](../decisions/0152-kernel-log-and-reset.md), [0213](../decisions/0213-power-capability.md)). It delivers interrupts as notifications [0167](../decisions/0167-interrupts-as-notifications.md), and on AArch64 it confines devices' interrupts through the GIC's ITS [0214](../decisions/0214-interrupt-remapping.md). Device memory and I/O ports get mapped through capabilities [0168](../decisions/0168-device-memory-and-ports.md), and each core is a kernel object [0165](../decisions/0165-core-objects.md). The IOMMU [0169](../decisions/0169-iommu-service.md) and the drivers [0172](../decisions/0172-device-manager.md) live outside the kernel, although it does hold back memory that a device may still reach [0170](../decisions/0170-dma-quarantine.md).

At its interfaces, the kernel accepts system calls only from the system library [0117](../decisions/0117-system-library.md). For IPC it has endpoints, reply objects and notifications ([0118](../decisions/0118-calls-and-notifications.md), [0120](../decisions/0120-reply-objects.md)), and it serves callers waiting on an endpoint in priority order [0220](../decisions/0220-endpoint-priority-order.md). A wakeup enters the scheduler only when the woken thread may run first [0219](../decisions/0219-wakeup-preemption.md). Capability spaces are radix tables, and programs pay for them [0126](../decisions/0126-capability-space-radix-table.md). Core sharing [0135](../decisions/0135-core-sharing-from-scheduler.md), address-space regions [0138](../decisions/0138-pager-keeps-reservations.md) and accounts [0140](../decisions/0140-accounts-capability-pair.md) are all kept out of it.

## Open questions

None remain open in this document.
