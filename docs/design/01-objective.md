# 01. Objective, scope and trust

- **Day:** 1 (Wed 2026-10-14)
- **Status:** In progress

## Covers

The purpose of the OS, its scope, the threat model, and what the system trusts.

## Decided

- [0001](../decisions/0001-general-purpose-os.md) General-purpose production OS
- [0002](../decisions/0002-kernel-mode-verified-execution.md) All code runs in kernel mode, isolated by verification (superseded by [0014](../decisions/0014-microkernel.md))
- [0003](../decisions/0003-rust-std-no-libc.md) Rust is the system language; the Rust standard library replaces libc (amended by [0082](../decisions/0082-corrections.md))
- [0004](../decisions/0004-not-unix.md) Not Unix-like: `target_family` is not `unix`
- [0008](../decisions/0008-architecture-neutral-design.md) Architecture-neutral design (amended by [0082](../decisions/0082-corrections.md))
- [0009](../decisions/0009-x86-64-first.md) x86_64 is the first architecture
- [0010](../decisions/0010-64-bit-only.md) 64-bit architectures only
- [0011](../decisions/0011-aarch64-second.md) AArch64 is the second architecture
- [0014](../decisions/0014-microkernel.md) Microkernel: a minimal, audited kernel in kernel mode; everything else in user mode (amended by [0082](../decisions/0082-corrections.md) and [0169](../decisions/0169-iommu-service.md))
- [0015](../decisions/0015-clean-apis.md) APIs and ABIs are designed for this system: inspired by others, never bound to them
- [0016](../decisions/0016-threat-model-scope.md) Threat model: speculative leaks between programs and denial of service are in scope
- [0019](../decisions/0019-kernel-minimality.md) Kernel minimality: anything that can reasonably run in user mode does
- [0045](../decisions/0045-aarch64-with-first-user-program.md) AArch64 is brought up together with the first user-mode program
- [0046](../decisions/0046-scope-of-0008.md) Scope of 0008: the OS's own code, including platform-specific code (amended by [0152](../decisions/0152-kernel-log-and-reset.md))
- [0116](../decisions/0116-name-thisos.md) The OS is named thisOS

## Open questions

- **Kernel contents.** Under [0019](../decisions/0019-kernel-minimality.md), the kernel keeps only what can't reasonably run in user mode, and each mechanism is judged on its own.
  - The starting point is seL4's set: address spaces, threads and scheduling, IPC, interrupt delivery and capabilities.
  - Already moved out: physical memory management [0025](../decisions/0025-user-mode-memory.md), page-fault handling [0027](../decisions/0027-user-mode-pagers.md), building page tables [0052](../decisions/0052-user-mode-page-tables.md), program loading [0031](../decisions/0031-user-mode-loader.md), most hardware discovery [0032](../decisions/0032-minimal-hardware-discovery.md) and scheduling policy [0088](../decisions/0088-user-mode-scheduler.md).
  - Settled for the kernel itself: it has no heap [0068](../decisions/0068-no-kernel-heap.md), it has one stack per core [0060](../decisions/0060-kernel-stack-per-core.md), it never reads program data [0072](../decisions/0072-registers-only.md), it runs with interrupts off between interruption points [0089](../decisions/0089-interrupts-off-in-kernel.md), and its guards against touching or running program memory are always on [0075](../decisions/0075-user-guards-always-on.md).
  - Settled for hardware: it reads the interrupt controller and the timer from the firmware tables [0151](../decisions/0151-boot-info-from-crate.md), writes its log to a serial port and resets the machine itself [0152](../decisions/0152-kernel-log-and-reset.md), delivers interrupts as notifications [0167](../decisions/0167-interrupts-as-notifications.md), and maps device memory and I/O ports through capabilities [0168](../decisions/0168-device-memory-and-ports.md). Each core is a kernel object [0165](../decisions/0165-core-objects.md). The IOMMU [0169](../decisions/0169-iommu-service.md) and the drivers [0172](../decisions/0172-device-manager.md) stay out of it, though the kernel holds back memory that a device may still reach [0170](../decisions/0170-dma-quarantine.md).
  - Settled for its interfaces: it accepts system calls only from the system library [0117](../decisions/0117-system-library.md), its IPC objects are endpoints, reply objects and notifications ([0118](../decisions/0118-calls-and-notifications.md), [0120](../decisions/0120-reply-objects.md)), and capability spaces are radix tables that programs pay for [0126](../decisions/0126-capability-space-radix-table.md). Core sharing [0135](../decisions/0135-core-sharing-from-scheduler.md), address-space regions [0138](../decisions/0138-pager-keeps-reservations.md) and accounts [0140](../decisions/0140-accounts-capability-pair.md) stay out of it.
