# 01. Objective, scope and trust

- **Day:** 1 (Wed 2026-10-14)
- **Status:** In progress

## Covers

The purpose of the OS, its scope, the threat model, and what the system trusts.

## Decided

- [0001](../decisions/0001-general-purpose-os.md) General-purpose production OS
- [0002](../decisions/0002-kernel-mode-verified-execution.md) All code runs in kernel mode, isolated by verification (superseded by [0014](../decisions/0014-microkernel.md))
- [0003](../decisions/0003-rust-std-no-libc.md) Rust is the system language; the Rust standard library replaces libc
- [0004](../decisions/0004-not-unix.md) Not Unix-like
- [0008](../decisions/0008-architecture-neutral-design.md) Architecture-neutral design
- [0009](../decisions/0009-x86-64-first.md) x86_64 is the first architecture
- [0010](../decisions/0010-64-bit-only.md) 64-bit architectures only
- [0011](../decisions/0011-aarch64-second.md) AArch64 is the second architecture
- [0014](../decisions/0014-microkernel.md) Microkernel: a minimal, audited kernel in kernel mode; everything else in user mode
- [0015](../decisions/0015-clean-apis.md) APIs and ABIs are designed for this system: inspired by others, never bound to them
- [0016](../decisions/0016-threat-model-scope.md) Threat model: speculative leaks between programs and denial of service are in scope
- [0019](../decisions/0019-kernel-minimality.md) Kernel minimality: anything that can reasonably run in user mode does
- [0045](../decisions/0045-aarch64-with-first-user-program.md) AArch64 is brought up together with the first user-mode program
- [0046](../decisions/0046-scope-of-0008.md) Scope of 0008: the OS's own code, including platform-specific code

## Open questions

- **Kernel contents.** Under [0019](../decisions/0019-kernel-minimality.md), the kernel keeps only what can't reasonably run in user mode, and each mechanism is judged on its own.
  - The starting point is seL4's set: address spaces, threads and scheduling, IPC, interrupt delivery and capabilities.
  - Even within those, scheduling policy may move to user mode.
  - Already moved out: physical memory management [0025](../decisions/0025-user-mode-memory.md), page-fault handling [0027](../decisions/0027-user-mode-pagers.md), building page tables [0052](../decisions/0052-user-mode-page-tables.md), program loading [0031](../decisions/0031-user-mode-loader.md) and most hardware discovery [0032](../decisions/0032-minimal-hardware-discovery.md).
  - Settled for the kernel itself: it has no heap [0068](../decisions/0068-no-kernel-heap.md), it has one stack per core [0060](../decisions/0060-kernel-stack-per-core.md), it never reads program data [0072](../decisions/0072-registers-only.md), and its guards against touching or running program memory are always on [0075](../decisions/0075-user-guards-always-on.md).
- **Project name.** Deferred: it isn't needed until the first crates are created. It also becomes the OS name in the Rust target.
