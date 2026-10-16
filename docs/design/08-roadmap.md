# 08. Implementation roadmap, engineering practices and freeze

- **Day:** 8 (Tue 2026-10-20)
- **Status:** Not started

## Covers

The exact plan for building thisOS: the order in which every part of the design is implemented, what each step delivers, how each step is tested, and when it counts as done. It also decides the engineering practices the whole build follows: modularity, the repository layout, testing and CI, review, and coding and documentation standards. This plan replaces the rough roadmap in the [README](../../README.md). At the end of the day, the design and the roadmap freeze together.

## Decided

Decisions that already fix part of the order or the practices:

- [0006](../decisions/0006-non-rust-software.md) Non-Rust software is supported later; route still open (amended by [0082](../decisions/0082-corrections.md))
- [0008](../decisions/0008-architecture-neutral-design.md) Architecture-neutral design (amended by [0082](../decisions/0082-corrections.md))
- [0009](../decisions/0009-x86-64-first.md) x86_64 is the first architecture
- [0018](../decisions/0018-kernel-assurance.md) Kernel assurance: review, tests and fuzzing from the start; formal proofs later
- [0019](../decisions/0019-kernel-minimality.md) Kernel minimality: anything that can reasonably run in user mode does
- [0045](../decisions/0045-aarch64-with-first-user-program.md) AArch64 is brought up together with the first user-mode program
- [0046](../decisions/0046-scope-of-0008.md) Scope of 0008: the OS's own code, including platform-specific code
- [0061](../decisions/0061-no-randomization-yet.md) Nothing is randomized at first, and the design keeps randomization possible
- [0068](../decisions/0068-no-kernel-heap.md) The kernel has no heap (amended by [0097](../decisions/0097-user-mode-starts-cores.md))
- [0077](../decisions/0077-swap-in-default-pager.md) Swap is a feature of the default pager; it comes later and is encrypted (amended by [0082](../decisions/0082-corrections.md))
- [0079](../decisions/0079-pointer-tagging-off.md) Hardware pointer tagging stays off for now
- [0148](../decisions/0148-freeze-covers-rules.md) The freeze covers the interface design, and each interface's version 1 is fixed when it's released

## Open questions

- **Steps and their order.** The steps from an empty repository to a working system, in the order their dependencies need. Every part of the design needs a place:
  - the boot path and the kernel's base: log output and panics [0110](../decisions/0110-kernel-panic.md), entering and leaving the kernel, the guards [0075](../decisions/0075-user-guards-always-on.md) and the mitigations for CPU flaws [0100](../decisions/0100-cpu-flaw-mitigations.md);
  - memory: memory capabilities, kernel objects and the object window [0067](../decisions/0067-object-window-offset.md), [0096](../decisions/0096-object-window-lazy-unmap.md), page-table objects [0052](../decisions/0052-user-mode-page-tables.md), clearing [0051](../decisions/0051-kernel-clears-memory.md), revoke [0053](../decisions/0053-revoke-destroys-derived.md) and the execute right [0054](../decisions/0054-kernel-enforces-execute.md);
  - threads: strict priorities [0090](../decisions/0090-priorities-in-kernel.md), the timer deadline [0094](../decisions/0094-one-deadline-per-core.md), context switching [0080](../decisions/0080-predictor-flush-both-ways.md), [0101](../decisions/0101-eager-fpu-state.md), interruption points [0089](../decisions/0089-interrupts-off-in-kernel.md), the kernel lock [0095](../decisions/0095-one-kernel-lock.md) and starting cores [0097](../decisions/0097-user-mode-starts-cores.md);
  - system calls and IPC: the system library [0117](../decisions/0117-system-library.md), registers only [0072](../decisions/0072-registers-only.md), the `sysret` guard [0125](../decisions/0125-sysret-guard.md), endpoints, reply objects and notifications ([0118](../decisions/0118-calls-and-notifications.md), [0120](../decisions/0120-reply-objects.md), [0122](../decisions/0122-bound-notifications.md)), passing capabilities [0123](../decisions/0123-capabilities-in-messages.md), direct switches on calls [0091](../decisions/0091-direct-switch-on-calls.md) and helping [0121](../decisions/0121-callers-help-busy-services.md), run-time reports [0092](../decisions/0092-scheduler-keeps-time.md), [0134](../decisions/0134-scheduler-interface.md), and faults sent to pagers and exception handlers [0081](../decisions/0081-pager-per-thread.md), [0102](../decisions/0102-exception-handler.md), [0106](../decisions/0106-pager-passes-on-faults.md);
  - capabilities: capability spaces [0126](../decisions/0126-capability-space-radix-table.md), rights and badges [0128](../decisions/0128-rights-and-badges.md), and revoke [0129](../decisions/0129-revoke-any-capability.md);
  - the interface tools: the IDL compiler and the compatibility checker ([0142](../decisions/0142-own-idl.md), [0143](../decisions/0143-add-only-versioning.md)), and when each interface's version 1 is released [0148](../decisions/0148-freeze-covers-rules.md);
  - the first user-mode programs: the root task [0024](../decisions/0024-root-task.md), the memory service and the default pager [0071](../decisions/0071-three-memory-programs.md), the scheduler [0088](../decisions/0088-user-mode-scheduler.md), the loader [0031](../decisions/0031-user-mode-loader.md), [0107](../decisions/0107-loader-writes-startup-data.md), the time service [0112](../decisions/0112-time-service.md), and supervisors [0104](../decisions/0104-supervisor.md);
  - hardware discovery [0032](../decisions/0032-minimal-hardware-discovery.md) and drivers;
  - storage: the filesystem [0030](../decisions/0030-filesystem-decides-access.md), the package store [0041](../decisions/0041-immutable-packages.md), signing [0040](../decisions/0040-signed-programs-owner-policy.md), and the system image and its updates [0042](../decisions/0042-updates.md);
  - login and users [0029](../decisions/0029-users-as-capabilities.md);
  - std, in its stages [0003](../decisions/0003-rust-std-no-libc.md), with its panic strategy [0109](../decisions/0109-panic-strategy.md).
- **What each step delivers.** For each step: the modules it builds, the decisions it implements, and the interfaces it builds.
- **When a step is done.** The tests that must pass, the architectures they must pass on [0045](../decisions/0045-aarch64-with-first-user-program.md), and the review a step needs before the next one starts [0018](../decisions/0018-kernel-assurance.md).
- **Coverage of the design.** How every Accepted decision is mapped to the step that implements it and the tests that check it, so that nothing in the design is left unbuilt or untested.
- **Modularity.** How the code is divided into modules with narrow interfaces: the kernel's internal parts, the architecture- and platform-specific modules and where they're allowed to live ([0008](../decisions/0008-architecture-neutral-design.md), [0046](../decisions/0046-scope-of-0008.md)), each user-mode service, and the libraries they share, such as the page-table library [0052](../decisions/0052-user-mode-page-tables.md). It also covers the repository layout.
- **Testing.** Which kinds of tests each step has, such as unit tests and tests that boot in QEMU. A QEMU test harness, and CI that builds and tests every supported architecture, x86_64 and AArch64 from the first user-mode program on [0045](../decisions/0045-aarch64-with-first-user-program.md). How regressions are kept out.
- **Review and kernel assurance.** Review rules for the kernel and the OS's own services, the kernel's test coverage targets and fuzzing setup [0018](../decisions/0018-kernel-assurance.md), and the longest stretch allowed between the kernel's interruption points [0089](../decisions/0089-interrupts-off-in-kernel.md). How kernel minimality is enforced: review rules, a size budget, or both [0019](../decisions/0019-kernel-minimality.md).
- **Coding and documentation standards.**
- **Estimates and checkpoints.** How long each step is expected to take, and the points where progress is checked against the plan.
- **Changing the plan.** What happens when building turns up something the plan or the design missed. This ties to [07-engineering](07-engineering.md)'s question on an errata process.
- **Later work.** Where each deferred item goes: non-Rust software [0006](../decisions/0006-non-rust-software.md), randomization [0061](../decisions/0061-no-randomization-yet.md), swap [0077](../decisions/0077-swap-in-default-pager.md), pointer tagging [0079](../decisions/0079-pointer-tagging-off.md), formal proofs [0018](../decisions/0018-kernel-assurance.md), and getting the Rust target accepted upstream.
- **The freeze.** At the end of Day 8, the design and the roadmap freeze together.
