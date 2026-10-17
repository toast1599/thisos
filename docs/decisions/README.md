# Decision log

Each decision gets one record, numbered in order. A record covers the context, the options that were considered, the decision itself, and its consequences.

## Status values

- **Accepted**: approved by the maintainer. Binding.
- **Proposed**: written up but not yet approved. Not binding.
- **Superseded**: replaced by a later record, which it links to.
- **Accepted, amended by**: binding, except for the lines that a later record corrects. The status links to that record.

Until the freeze on 2026-10-20, an Accepted decision can still be changed. Changing one means writing a new record that supersedes it, or one that amends some of its lines. Records are never edited after the fact; only their status changes.

Open questions are listed in the [design documents](../design/), grouped by topic.

## Records

| # | Decision | Status | Date |
|---|---|---|---|
| [0001](0001-general-purpose-os.md) | General-purpose production OS | Accepted | 2026-10-14 |
| [0002](0002-kernel-mode-verified-execution.md) | All code runs in kernel mode, isolated by verification | Superseded by [0014](0014-microkernel.md) | 2026-10-14 |
| [0003](0003-rust-std-no-libc.md) | Rust is the system language; the Rust standard library replaces libc | Accepted, amended by [0082](0082-corrections.md) | 2026-10-14 |
| [0004](0004-not-unix.md) | Not Unix-like: `target_family` is not `unix` | Accepted | 2026-10-14 |
| [0005](0005-unsafe-code-policy.md) | Unsafe code exists only in one closed, audited trusted set | Superseded by [0017](0017-unsafe-code-kernel-and-services.md) | 2026-10-14 |
| [0006](0006-non-rust-software.md) | Non-Rust software is supported later; route still open | Accepted, amended by [0082](0082-corrections.md) | 2026-10-14 |
| [0007](0007-abi-calls-and-interfaces.md) | ABI: C calling convention, interfaces defined in an IDL, add-only versioning | Superseded by [0144](0144-c-calls-and-what-crosses.md) (never accepted) | 2026-10-14 |
| [0008](0008-architecture-neutral-design.md) | Architecture-neutral design | Accepted, amended by [0082](0082-corrections.md) | 2026-10-14 |
| [0009](0009-x86-64-first.md) | x86_64 is the first architecture | Accepted | 2026-10-14 |
| [0010](0010-64-bit-only.md) | 64-bit architectures only | Accepted | 2026-10-14 |
| [0011](0011-aarch64-second.md) | AArch64 is the second architecture | Accepted | 2026-10-15 |
| [0012](0012-no-memory-cap.md) | No fixed limit on program memory | Accepted | 2026-10-15 |
| [0013](0013-isolation-domains.md) | Isolation domains: one shared system domain, one checked address space per program | Superseded by [0014](0014-microkernel.md) (never accepted) | 2026-10-15 |
| [0014](0014-microkernel.md) | Microkernel: a minimal, audited kernel in kernel mode; everything else in user mode | Accepted, amended by [0082](0082-corrections.md) and [0169](0169-iommu-service.md) | 2026-10-15 |
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
| [0027](0027-user-mode-pagers.md) | Page faults are handled by user-mode pagers | Accepted, amended by [0081](0081-pager-per-thread.md) | 2026-10-15 |
| [0028](0028-namespace-at-spawn.md) | Programs find services through a namespace handed over at spawn | Accepted | 2026-10-15 |
| [0029](0029-users-as-capabilities.md) | A user is the set of capabilities a login service grants | Accepted | 2026-10-15 |
| [0030](0030-filesystem-decides-access.md) | The filesystem service decides who may open stored data | Accepted | 2026-10-15 |
| [0031](0031-user-mode-loader.md) | The program loader runs in user mode, and starting programs needs a capability | Accepted | 2026-10-15 |
| [0032](0032-minimal-hardware-discovery.md) | The kernel reads only the hardware information it needs to start | Accepted, amended by [0152](0152-kernel-log-and-reset.md) | 2026-10-15 |
| [0033](0033-cpu-fairness.md) | CPU fairness: fair sharing within each priority, and priorities as capabilities | Accepted, amended by [0084](0084-shares-belong-to-accounts.md) | 2026-10-15 |
| [0034](0034-smt-trusted-pairs.md) | Only programs that trust each other share a CPU core | Accepted | 2026-10-15 |
| [0035](0035-predictor-flush-on-request.md) | Branch predictors are flushed only on request | Superseded by [0080](0080-predictor-flush-both-ways.md) | 2026-10-15 |
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
| [0046](0046-scope-of-0008.md) | Scope of 0008: the OS's own code, including platform-specific code | Accepted, amended by [0152](0152-kernel-log-and-reset.md) | 2026-10-15 |
| [0047](0047-kernel-mapping.md) | The kernel stays mapped in every address space unless the CPU requires otherwise | Accepted | 2026-10-15 |
| [0048](0048-reserve-on-grant.md) | Memory is reserved when it's granted; the system never promises more than it has | Accepted | 2026-10-15 |
| [0049](0049-memory-fair-share.md) | When memory runs short, it's shared fairly within each memory priority | Accepted, amended by [0084](0084-shares-belong-to-accounts.md) | 2026-10-15 |
| [0050](0050-clients-pay.md) | Clients pay for the state services keep for them | Accepted | 2026-10-15 |
| [0051](0051-kernel-clears-memory.md) | The kernel clears memory before it's made into anything new | Accepted | 2026-10-15 |
| [0052](0052-user-mode-page-tables.md) | Page tables are kernel objects that user mode builds | Accepted | 2026-10-15 |
| [0053](0053-revoke-destroys-derived.md) | Revoking memory destroys everything made from it | Accepted, amended by [0170](0170-dma-quarantine.md) | 2026-10-15 |
| [0054](0054-kernel-enforces-execute.md) | The kernel enforces the right to make memory executable | Accepted | 2026-10-15 |
| [0055](0055-kernel-maps-only-objects.md) | The kernel maps its own memory and kernel objects, not all of physical memory | Accepted, amended by [0096](0096-object-window-lazy-unmap.md) | 2026-10-15 |
| [0056](0056-pager-chosen-at-spawn.md) | A program's pager is chosen when it's spawned | Accepted, amended by [0081](0081-pager-per-thread.md) | 2026-10-15 |
| [0057](0057-pager-fault-chain.md) | A pager's own faults go to its pager, up a chain that ends at the root task | Accepted | 2026-10-15 |
| [0058](0058-shared-and-handed-over.md) | Programs share bulk data through regions that are shared or handed over | Accepted | 2026-10-15 |
| [0059](0059-caller-allocates.md) | Memory that crosses an interface is allocated by the caller, and its use moves only by handover | Accepted | 2026-10-15 |
| [0060](0060-kernel-stack-per-core.md) | The kernel has one stack per CPU core, and its worst-case depth is checked at build time | Accepted | 2026-10-15 |
| [0061](0061-no-randomization-yet.md) | Nothing is randomized at first, and the design keeps randomization possible | Accepted | 2026-10-15 |
| [0062](0062-memory-priority-capability.md) | Memory priority is a capability of its own | Accepted | 2026-10-15 |
| [0063](0063-widest-addressing.md) | The kernel turns on the widest virtual addressing the CPU offers | Accepted | 2026-10-15 |
| [0064](0064-memory-belongs-to-payer.md) | Memory belongs to the program it was granted to; lending, sharing and handover move only access | Accepted, amended by [0084](0084-shares-belong-to-accounts.md) | 2026-10-15 |
| [0065](0065-lost-memory-is-an-error.md) | Touching memory that was taken back is an error, not a crash | Accepted | 2026-10-15 |
| [0066](0066-kernel-upper-half.md) | The kernel sits in the upper half of every address space, and programs in the lower half | Accepted | 2026-10-15 |
| [0067](0067-object-window-offset.md) | Kernel objects are mapped at their physical address plus a fixed offset | Accepted, amended by [0096](0096-object-window-lazy-unmap.md) | 2026-10-15 |
| [0068](0068-no-kernel-heap.md) | The kernel has no heap | Accepted, amended by [0097](0097-user-mode-starts-cores.md) | 2026-10-15 |
| [0069](0069-all-page-sizes.md) | The kernel supports every page size the hardware offers | Accepted | 2026-10-15 |
| [0070](0070-position-independent-programs.md) | Programs are built position-independent | Accepted | 2026-10-15 |
| [0071](0071-three-memory-programs.md) | The root task, the memory service and the default pager are three separate programs | Accepted | 2026-10-15 |
| [0072](0072-registers-only.md) | System calls and IPC carry data only in registers; the kernel never reads program data | Accepted | 2026-10-15 |
| [0073](0073-kernel-layout-in-implementation.md) | The kernel's internal layout is chosen during implementation | Accepted | 2026-10-15 |
| [0074](0074-low-64k-unmapped.md) | The lowest 64 KiB of every program's half stays unmapped | Accepted | 2026-10-15 |
| [0075](0075-user-guards-always-on.md) | The kernel's guards against touching or running program memory are always on | Accepted | 2026-10-15 |
| [0076](0076-memory-notice-deadline.md) | Programs over their fair share get a notice, then a deadline | Accepted | 2026-10-15 |
| [0077](0077-swap-in-default-pager.md) | Swap is a feature of the default pager; it comes later and is encrypted | Accepted, amended by [0082](0082-corrections.md) | 2026-10-15 |
| [0078](0078-low-addresses-on-request.md) | Programs that need low addresses ask for them | Accepted | 2026-10-15 |
| [0079](0079-pointer-tagging-off.md) | Hardware pointer tagging stays off for now | Accepted | 2026-10-15 |
| [0080](0080-predictor-flush-both-ways.md) | Branch predictors are flushed when switching to or away from a program that asks | Accepted | 2026-10-16 |
| [0081](0081-pager-per-thread.md) | Page faults go to the faulting thread's pager | Accepted | 2026-10-16 |
| [0082](0082-corrections.md) | Corrections to 0003, 0006, 0008, 0014 and 0077 | Accepted | 2026-10-16 |
| [0083](0083-program-is-address-space.md) | To the kernel, a program is an address space and the threads bound to it | Accepted | 2026-10-16 |
| [0084](0084-shares-belong-to-accounts.md) | Fair shares of CPU time and memory belong to accounts | Accepted, amended by [0092](0092-scheduler-keeps-time.md) | 2026-10-16 |
| [0085](0085-clients-pay-cpu-time.md) | Clients pay for the CPU time services spend on them | Accepted, amended by [0121](0121-callers-help-busy-services.md) | 2026-10-16 |
| [0086](0086-priorities-per-program-shares-per-account.md) | Priorities stay per program, and each priority is shared by account | Accepted | 2026-10-16 |
| [0087](0087-thread-priorities.md) | A thread runs at any CPU priority up to its program's | Accepted | 2026-10-16 |
| [0088](0088-user-mode-scheduler.md) | Scheduling policy runs in user mode | Accepted, amended by [0090](0090-priorities-in-kernel.md) | 2026-10-16 |
| [0089](0089-interrupts-off-in-kernel.md) | The kernel runs with interrupts off between its interruption points | Accepted | 2026-10-16 |
| [0090](0090-priorities-in-kernel.md) | Strict priorities stay in the kernel, and the scheduler orders threads within each priority | Accepted | 2026-10-16 |
| [0091](0091-direct-switch-on-calls.md) | Calls and replies switch directly between client and service | Accepted | 2026-10-16 |
| [0092](0092-scheduler-keeps-time.md) | The scheduler keeps the CPU time books, from the kernel's reports | Accepted | 2026-10-16 |
| [0093](0093-scheduler-per-core.md) | The scheduler has a thread on each core, which decides for that core | Accepted | 2026-10-16 |
| [0094](0094-one-deadline-per-core.md) | Each core has one timer deadline, set by its scheduler thread | Accepted | 2026-10-16 |
| [0095](0095-one-kernel-lock.md) | One lock guards the kernel on multicore | Accepted | 2026-10-16 |
| [0096](0096-object-window-lazy-unmap.md) | The object window keeps memory mapped until it becomes a program's pages | Accepted | 2026-10-16 |
| [0097](0097-user-mode-starts-cores.md) | User mode finds the CPUs, and the root task asks the kernel to start each one | Accepted | 2026-10-16 |
| [0098](0098-core-sharing-capability.md) | Programs share a core only if both hold the same core-sharing capability | Accepted | 2026-10-16 |
| [0099](0099-kernel-beside-sibling.md) | The kernel doesn't pause a sibling hardware thread when it runs | Accepted | 2026-10-16 |
| [0100](0100-cpu-flaw-mitigations.md) | The kernel applies each CPU's documented mitigations for speculative-execution flaws | Accepted | 2026-10-16 |
| [0101](0101-eager-fpu-state.md) | Floating-point and vector registers are saved and restored on every switch | Accepted | 2026-10-16 |
| [0102](0102-exception-handler.md) | Each thread names an exception handler besides its pager | Accepted | 2026-10-16 |
| [0103](0103-fault-without-handler.md) | A fault with nowhere to go suspends the thread, and in the root task halts the system | Accepted, amended by [0115](0115-root-task-fault-boot-setting.md) | 2026-10-16 |
| [0104](0104-supervisor.md) | Every program has a supervisor, named at spawn, that learns how it ended | Accepted | 2026-10-16 |
| [0105](0105-supervisor-reclaims.md) | A failed program's supervisor has its memory taken back, and decides on a restart | Accepted | 2026-10-16 |
| [0106](0106-pager-passes-on-faults.md) | A pager passes faults it won't resolve to the thread's exception handler | Accepted | 2026-10-16 |
| [0107](0107-loader-writes-startup-data.md) | The loader writes a new program's startup information into its memory | Accepted, amended by [0160](0160-root-task-starts-four.md) | 2026-10-16 |
| [0108](0108-startup-information.md) | What a new program receives when it's spawned | Accepted, amended by [0146](0146-startup-data-block.md), [0174](0174-random-service-and-seed.md), [0176](0176-paths-and-current-directory.md) and [0179](0179-command-copies-namespace.md) | 2026-10-16 |
| [0109](0109-panic-strategy.md) | Third-party programs unwind on panic by default, and the OS's own services abort | Accepted | 2026-10-16 |
| [0110](0110-kernel-panic.md) | A kernel panic is logged, then the machine halts or reboots by a boot setting | Accepted | 2026-10-16 |
| [0111](0111-resource-accounting.md) | Each service reports the resource use it already tracks | Accepted | 2026-10-16 |
| [0112](0112-time-service.md) | Programs read the time from a time service | Accepted | 2026-10-16 |
| [0113](0113-counter-capability.md) | Only programs holding a capability may read the CPU's time counter | Accepted | 2026-10-16 |
| [0114](0114-scheduler-priority.md) | Each core's scheduler thread runs above every priority it schedules | Accepted | 2026-10-16 |
| [0115](0115-root-task-fault-boot-setting.md) | A fault in the root task is handled like a kernel panic | Accepted | 2026-10-16 |
| [0116](0116-name-thisos.md) | The OS is named thisOS | Accepted | 2026-10-16 |
| [0117](0117-system-library.md) | Programs reach the kernel only through the system library | Accepted, amended by [0160](0160-root-task-starts-four.md) | 2026-10-17 |
| [0118](0118-calls-and-notifications.md) | IPC is synchronous calls through endpoints, with notifications for events | Accepted | 2026-10-17 |
| [0119](0119-message-eight-words.md) | One IPC message carries up to 8 words | Accepted | 2026-10-17 |
| [0120](0120-reply-objects.md) | Services reply through reply objects | Accepted | 2026-10-17 |
| [0121](0121-callers-help-busy-services.md) | A caller that finds its service busy lends it its priority and time | Accepted | 2026-10-17 |
| [0122](0122-bound-notifications.md) | A notification can be bound to a thread | Accepted | 2026-10-17 |
| [0123](0123-capabilities-in-messages.md) | A message carries up to four capabilities, each copied or moved | Accepted | 2026-10-17 |
| [0124](0124-large-calls-split.md) | Kernel calls whose data doesn't fit in registers are split, and the system library joins them | Accepted | 2026-10-17 |
| [0125](0125-sysret-guard.md) | The kernel never returns with `sysret` to an address outside the program half | Accepted | 2026-10-17 |
| [0126](0126-capability-space-radix-table.md) | A capability space is a radix table, and capabilities are named by index | Accepted | 2026-10-17 |
| [0127](0127-capability-space-per-program.md) | Each program has exactly one capability space | Accepted | 2026-10-17 |
| [0128](0128-rights-and-badges.md) | Capabilities carry a few rights, and endpoint and notification capabilities carry a badge | Accepted | 2026-10-17 |
| [0129](0129-revoke-any-capability.md) | Revoking any capability destroys every copy made from it | Accepted | 2026-10-17 |
| [0130](0130-handlers-changeable.md) | A thread's pager and exception handler can be changed through its control right | Accepted | 2026-10-17 |
| [0131](0131-namespace-path-prefixes.md) | A namespace maps path prefixes to capabilities | Accepted | 2026-10-17 |
| [0132](0132-events-notify-then-call.md) | An event is a notification, followed by a call for its details | Accepted | 2026-10-17 |
| [0133](0133-recovery-point-in-register.md) | Accessors keep their recovery point in a register | Accepted | 2026-10-17 |
| [0134](0134-scheduler-interface.md) | Threads run once the scheduler takes them on, and the kernel reports to it in registers | Accepted, amended by [0161](0161-root-task-before-scheduler.md) | 2026-10-17 |
| [0135](0135-core-sharing-from-scheduler.md) | The scheduler issues core-sharing capabilities | Accepted | 2026-10-17 |
| [0136](0136-predictor-flush-flag.md) | A program asks for predictor flushes with a flag on its address space | Accepted | 2026-10-17 |
| [0137](0137-memory-capability-per-program.md) | Each program has its own capability to the memory service, which names it as payer | Accepted, amended by [0164](0164-root-task-memory-pool.md) | 2026-10-17 |
| [0138](0138-pager-keeps-reservations.md) | Pagers keep a program's address-space reservations | Accepted | 2026-10-17 |
| [0139](0139-results-fill-and-report.md) | A result of unknown size fills what the caller provides, and reports what remains | Accepted | 2026-10-17 |
| [0140](0140-accounts-capability-pair.md) | An account is a pair of capabilities, one from the scheduler and one from the memory service | Accepted | 2026-10-17 |
| [0141](0141-identity-capability.md) | The login service gives each session an identity capability | Accepted | 2026-10-17 |
| [0142](0142-own-idl.md) | Interfaces are defined in our own IDL, modeled on FIDL | Accepted | 2026-10-17 |
| [0143](0143-add-only-versioning.md) | Interfaces only grow, and a tool checks every change | Accepted | 2026-10-17 |
| [0144](0144-c-calls-and-what-crosses.md) | The system library uses the C calling convention, and only data, capabilities and regions cross between programs | Accepted | 2026-10-17 |
| [0145](0145-std-over-native-api.md) | std is a layer over the native API | Accepted | 2026-10-17 |
| [0146](0146-startup-data-block.md) | Startup data is a versioned block defined in the IDL | Accepted | 2026-10-17 |
| [0147](0147-elf-declarations-only.md) | A program's ELF sections carry only its declarations | Accepted | 2026-10-17 |
| [0148](0148-freeze-covers-rules.md) | The freeze covers the interface design, and each interface's version 1 is fixed when it's released | Accepted | 2026-10-17 |
| [0149](0149-bootloader-crate-uefi.md) | The kernel is booted by the `bootloader` crate, through UEFI only | Accepted | 2026-10-17 |
| [0150](0150-bootloader-aarch64.md) | AArch64 support is added to the `bootloader` crate | Accepted | 2026-10-17 |
| [0151](0151-boot-info-from-crate.md) | The kernel starts from the crate's boot information | Accepted | 2026-10-17 |
| [0152](0152-kernel-log-and-reset.md) | The kernel writes its log to memory and a serial port, and resets the machine itself | Accepted | 2026-10-17 |
| [0153](0153-build-unpacks-ramdisk.md) | The build unpacks the root task and the system library into the ramdisk | Accepted | 2026-10-17 |
| [0154](0154-library-entry-table.md) | Programs call the system library through a table of entry points | Accepted | 2026-10-17 |
| [0155](0155-ramdisk-reaches-storage.md) | The ramdisk holds the programs needed to reach storage | Accepted | 2026-10-17 |
| [0156](0156-boot-loader-checks-image.md) | The boot loader checks the kernel and the ramdisk against the system image's signature | Accepted | 2026-10-17 |
| [0157](0157-boot-settings-in-ramdisk.md) | Boot settings live in the ramdisk's header | Accepted | 2026-10-17 |
| [0158](0158-image-try-count.md) | A new system image is tried a limited number of times, then the boot loader falls back | Accepted | 2026-10-17 |
| [0159](0159-root-task-boot-information.md) | The root task receives the usual startup data, with a block of boot information | Accepted | 2026-10-17 |
| [0160](0160-root-task-starts-four.md) | The root task starts four programs itself, and the rest from a boot manifest | Accepted | 2026-10-17 |
| [0161](0161-root-task-before-scheduler.md) | The root task runs alone until it starts the scheduler | Accepted | 2026-10-17 |
| [0162](0162-scheduler-above-priorities.md) | A core's scheduler thread runs above every priority, by a rule in the kernel | Accepted | 2026-10-17 |
| [0163](0163-services-hand-first-capabilities.md) | A service hands the first capabilities it issues to the program that started it | Accepted | 2026-10-17 |
| [0164](0164-root-task-memory-pool.md) | The root task keeps back a pool of memory for the programs it pages | Accepted | 2026-10-17 |
| [0165](0165-core-objects.md) | Each core is a kernel object, named by its capability | Accepted | 2026-10-17 |
| [0166](0166-core-revoke-offline.md) | Revoking a running core's memory takes the core offline | Accepted | 2026-10-17 |
| [0167](0167-interrupts-as-notifications.md) | Interrupts reach drivers as notifications | Accepted | 2026-10-17 |
| [0168](0168-device-memory-and-ports.md) | Device memory and I/O ports are capabilities of their own | Accepted | 2026-10-17 |
| [0169](0169-iommu-service.md) | A user-mode IOMMU service keeps the devices' I/O page tables | Accepted | 2026-10-17 |
| [0170](0170-dma-quarantine.md) | Memory mapped for a device is held back after a revoke, until the IOMMU service releases it | Accepted | 2026-10-17 |
| [0171](0171-no-iommu-trusted-drivers.md) | Without an IOMMU, the drivers of DMA-capable devices are trusted | Accepted | 2026-10-17 |
| [0172](0172-device-manager.md) | A device manager starts each driver with exactly its device's capabilities | Accepted | 2026-10-17 |
| [0173](0173-time-service-entry.md) | Programs reach the time service through the namespace entry `svc/time` | Accepted | 2026-10-17 |
| [0174](0174-random-service-and-seed.md) | Random numbers come from a random service, and every program starts with a seed | Accepted | 2026-10-17 |
| [0175](0175-rust-target-names.md) | The Rust targets are `x86_64-unknown-thisos` and `aarch64-unknown-thisos`, with no target family | Accepted | 2026-10-17 |
| [0176](0176-paths-and-current-directory.md) | Paths that start with `/` resolve through the namespace, and others through a current directory | Accepted | 2026-10-17 |
| [0177](0177-locks-on-parking.md) | std's locks are built on parking, and parking on notifications | Accepted | 2026-10-17 |
| [0178](0178-native-api-crate.md) | The native API is a separate crate | Accepted | 2026-10-17 |
| [0179](0179-command-copies-namespace.md) | std's `Command` gives a child a copy of the spawner's namespace by default | Accepted | 2026-10-17 |
| [0180](0180-streams-and-exit-status.md) | Standard streams use a byte-stream protocol, and a program ends with a code or an abnormal end | Accepted | 2026-10-17 |
| [0181](0181-network-service.md) | `std::net` uses a network service through the namespace | Accepted | 2026-10-17 |
| [0182](0182-all-of-std.md) | All of std is supported, in stages, and allocation failure aborts | Accepted | 2026-10-17 |
| [0183](0183-access-on-directories.md) | Stored access information lives on directories, as grants to stable user IDs | Accepted | 2026-10-17 |
| [0184](0184-shared-file-pages.md) | Read-only file pages are shared from the service's cache, which pays for them | Accepted | 2026-10-17 |
| [0185](0185-swap-partition.md) | Swap goes to a partition, through the block-storage service | Accepted | 2026-10-17 |
| [0186](0186-non-rust-route.md) | Non-Rust software runs on WebAssembly first, then natively on the native API | Accepted | 2026-10-17 |
