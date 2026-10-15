# Decision log

Each decision gets one record, numbered in order. A record covers the context, the options that were considered, the decision itself, and its consequences.

## Status values

- **Accepted**: approved by the maintainer. Binding.
- **Proposed**: written up but not yet approved. Not binding.
- **Superseded**: replaced by a later record, which it links to.

Until the freeze on 2026-10-20, an Accepted decision can still be changed. Changing one means writing a new record that supersedes it; records are never edited after the fact.

Open questions are listed in the [design documents](../design/), grouped by topic.

## Records

| # | Decision | Status | Date |
|---|---|---|---|
| [0001](0001-general-purpose-os.md) | General-purpose production OS | Accepted | 2026-10-14 |
| [0002](0002-kernel-mode-verified-execution.md) | All code runs in kernel mode, isolated by verification | Superseded by [0014](0014-microkernel.md) | 2026-10-14 |
| [0003](0003-rust-std-no-libc.md) | Rust is the system language; the Rust standard library replaces libc | Accepted | 2026-10-14 |
| [0004](0004-not-unix.md) | Not Unix-like: `target_family` is not `unix` | Accepted | 2026-10-14 |
| [0005](0005-unsafe-code-policy.md) | Unsafe code exists only in one closed, audited trusted set | Superseded by [0017](0017-unsafe-code-kernel-and-services.md) | 2026-10-14 |
| [0006](0006-non-rust-software.md) | Non-Rust software is supported later; route still open | Accepted | 2026-10-14 |
| [0007](0007-abi-calls-and-interfaces.md) | ABI: C calling convention, interfaces defined in an IDL, add-only versioning | Proposed | 2026-10-14 |
| [0008](0008-architecture-neutral-design.md) | Architecture-neutral design: only architecture-specific code relies on architecture-specific concepts | Accepted | 2026-10-14 |
| [0009](0009-x86-64-first.md) | x86_64 is the first architecture | Accepted | 2026-10-14 |
| [0010](0010-64-bit-only.md) | 64-bit architectures only | Accepted | 2026-10-14 |
| [0011](0011-aarch64-second.md) | AArch64 is the second architecture | Accepted | 2026-10-15 |
| [0012](0012-no-memory-cap.md) | No fixed limit on program memory | Accepted | 2026-10-15 |
| [0013](0013-isolation-domains.md) | Isolation domains: one shared system domain, one checked address space per program | Superseded by [0014](0014-microkernel.md) | 2026-10-15 |
| [0014](0014-microkernel.md) | Microkernel: a minimal, audited kernel in kernel mode; everything else in user mode | Accepted | 2026-10-15 |
| [0015](0015-clean-apis.md) | APIs and ABIs are designed for this system: inspired by others, never bound to them | Accepted | 2026-10-15 |
| [0016](0016-threat-model-scope.md) | Threat model: speculative leaks between programs and denial of service are in scope | Accepted | 2026-10-15 |
| [0017](0017-unsafe-code-kernel-and-services.md) | Unsafe code: one closed, audited set for the kernel and the OS's own services | Accepted | 2026-10-15 |
| [0018](0018-kernel-assurance.md) | Kernel assurance: review, tests and fuzzing from the start; formal proofs later | Accepted | 2026-10-15 |
| [0019](0019-kernel-minimality.md) | Kernel minimality: anything that can reasonably run in user mode does | Accepted | 2026-10-15 |
| [0020](0020-capabilities.md) | Authority comes from capabilities | Accepted | 2026-10-15 |
| [0021](0021-ipc-simple-upgradable.md) | IPC starts simple and modular, and is designed to be upgraded | Accepted | 2026-10-15 |
| [0022](0022-explicit-spawn.md) | Programs start other programs by explicit spawn | Accepted | 2026-10-15 |
| [0023](0023-no-quotas.md) | No quotas: a program's resources are limited only by the hardware and what's available | Accepted | 2026-10-15 |
| [0024](0024-root-task.md) | At boot, one root task receives every initial capability | Accepted | 2026-10-15 |
| [0025](0025-user-mode-memory.md) | Physical memory is managed from user mode | Accepted | 2026-10-15 |
| [0026](0026-programs-pay-kernel-objects.md) | Programs pay for their kernel objects with memory they hold | Accepted | 2026-10-15 |
| [0027](0027-user-mode-pagers.md) | Page faults are handled by user-mode pagers | Accepted | 2026-10-15 |
| [0028](0028-namespace-at-spawn.md) | Programs find services through a namespace handed over at spawn | Accepted | 2026-10-15 |
| [0029](0029-users-as-capabilities.md) | A user is the set of capabilities a login service grants | Accepted | 2026-10-15 |
| [0030](0030-filesystem-decides-access.md) | The filesystem service decides who may open stored data | Accepted | 2026-10-15 |
| [0031](0031-user-mode-loader.md) | The program loader runs in user mode, and starting programs needs a capability | Accepted | 2026-10-15 |
| [0032](0032-minimal-hardware-discovery.md) | The kernel reads only the hardware information it needs to start | Accepted | 2026-10-15 |
| [0033](0033-cpu-fairness.md) | CPU fairness: fair sharing within each priority, and priorities as capabilities | Accepted | 2026-10-15 |
| [0034](0034-smt-trusted-pairs.md) | Only programs that trust each other share a CPU core | Accepted | 2026-10-15 |
| [0035](0035-predictor-flush-on-request.md) | Branch predictors are flushed only on request | Accepted | 2026-10-15 |
| [0036](0036-ipc-message-max.md) | One IPC message has a small fixed maximum size; larger data goes through shared memory | Accepted | 2026-10-15 |
| [0037](0037-kernel-toolchain.md) | The kernel is built with one pinned, audited toolchain | Accepted | 2026-10-15 |
| [0038](0038-elf-container.md) | Programs ship as ELF, used only as a file container | Accepted | 2026-10-15 |
| [0039](0039-native-per-arch.md) | Programs ship as native code, built separately for each architecture | Accepted | 2026-10-15 |
| [0040](0040-signed-programs-owner-policy.md) | Programs are signed, and the device owner's policy decides which signers may run | Accepted | 2026-10-15 |
| [0041](0041-immutable-packages.md) | Installed programs are immutable packages named by their content hash | Accepted | 2026-10-15 |
| [0042](0042-updates.md) | The system updates atomically as one image; third-party programs update package by package | Accepted | 2026-10-15 |
| [0043](0043-executable-memory-capability.md) | Making memory executable at run time needs a capability | Accepted | 2026-10-15 |
| [0044](0044-system-image-is-os-shipped.md) | Everything in the system image counts as shipped by the OS | Accepted | 2026-10-15 |
| [0045](0045-aarch64-with-first-user-program.md) | AArch64 is brought up together with the first user-mode program | Accepted | 2026-10-15 |
| [0046](0046-scope-of-0008.md) | Scope of 0008: the OS's own code, including platform-specific code | Accepted | 2026-10-15 |
| [0047](0047-kernel-mapping.md) | The kernel stays mapped in every address space unless the CPU requires otherwise | Accepted | 2026-10-15 |
| [0048](0048-reserve-on-grant.md) | Memory is reserved when it's granted; the system never promises more than it has | Accepted | 2026-10-15 |
| [0049](0049-memory-fair-share.md) | When memory runs short, it's shared fairly within each memory priority | Accepted | 2026-10-15 |
| [0050](0050-clients-pay.md) | Clients pay for the state services keep for them | Accepted | 2026-10-15 |
| [0051](0051-kernel-clears-memory.md) | The kernel clears memory before it's made into anything new | Accepted | 2026-10-15 |
| [0052](0052-user-mode-page-tables.md) | Page tables are kernel objects that user mode builds | Accepted | 2026-10-15 |
| [0053](0053-revoke-destroys-derived.md) | Revoking memory destroys everything made from it | Accepted | 2026-10-15 |
| [0054](0054-kernel-enforces-execute.md) | The kernel enforces the right to make memory executable | Accepted | 2026-10-15 |
| [0055](0055-kernel-maps-only-objects.md) | The kernel maps its own memory and kernel objects, not all of physical memory | Accepted | 2026-10-15 |
| [0056](0056-pager-chosen-at-spawn.md) | A program's pager is chosen when it's spawned | Accepted | 2026-10-15 |
| [0057](0057-pager-fault-chain.md) | A pager's own faults go to its pager, up a chain that ends at the root task | Accepted | 2026-10-15 |
| [0058](0058-shared-and-handed-over.md) | Programs share bulk data through regions that are shared or handed over | Accepted | 2026-10-15 |
| [0059](0059-caller-allocates.md) | Memory that crosses an interface is allocated by the caller, and its use moves only by handover | Accepted | 2026-10-15 |
| [0060](0060-kernel-stack-per-core.md) | The kernel has one stack per CPU core, and its worst-case depth is checked at build time | Accepted | 2026-10-15 |
| [0061](0061-no-randomization-yet.md) | Nothing is randomized at first, and the design keeps randomization possible | Accepted | 2026-10-15 |
| [0062](0062-memory-priority-capability.md) | Memory priority is a capability of its own | Accepted | 2026-10-15 |
| [0063](0063-widest-addressing.md) | The kernel turns on the widest virtual addressing the CPU offers | Accepted | 2026-10-15 |
| [0064](0064-memory-belongs-to-payer.md) | Memory belongs to the program it was granted to; lending, sharing and handover move only access | Accepted | 2026-10-15 |
| [0065](0065-lost-memory-is-an-error.md) | Touching memory that was taken back is an error, not a crash | Accepted | 2026-10-15 |
| [0066](0066-kernel-upper-half.md) | The kernel sits in the upper half of every address space, and programs in the lower half | Accepted | 2026-10-15 |
| [0067](0067-object-window-offset.md) | Kernel objects are mapped at their physical address plus a fixed offset | Accepted | 2026-10-15 |
| [0068](0068-no-kernel-heap.md) | The kernel has no heap | Accepted | 2026-10-15 |
| [0069](0069-all-page-sizes.md) | The kernel supports every page size the hardware offers | Accepted | 2026-10-15 |
| [0070](0070-position-independent-programs.md) | Programs are built position-independent | Accepted | 2026-10-15 |
| [0071](0071-three-memory-programs.md) | The root task, the memory service and the default pager are three separate programs | Accepted | 2026-10-15 |
| [0072](0072-registers-only.md) | System calls and IPC carry data only in registers; the kernel never reads program data | Accepted | 2026-10-15 |
| [0073](0073-kernel-layout-in-implementation.md) | The kernel's internal layout is chosen during implementation | Accepted | 2026-10-15 |
| [0074](0074-low-64k-unmapped.md) | The lowest 64 KiB of every program's half stays unmapped | Accepted | 2026-10-15 |
| [0075](0075-user-guards-always-on.md) | The kernel's guards against touching or running program memory are always on | Accepted | 2026-10-15 |
| [0076](0076-memory-notice-deadline.md) | Programs over their fair share get a notice, then a deadline | Accepted | 2026-10-15 |
| [0077](0077-swap-in-default-pager.md) | Swap is a feature of the default pager; it comes later and is encrypted | Accepted | 2026-10-15 |
| [0078](0078-low-addresses-on-request.md) | Programs that need low addresses ask for them | Accepted | 2026-10-15 |
| [0079](0079-pointer-tagging-off.md) | Hardware pointer tagging stays off for now | Accepted | 2026-10-15 |
