# 0227. The roadmap has 24 steps in 7 phases, each after the steps it needs

- **Status:** Accepted, amended by [0271](0271-corrections.md)
- **Date:** 2026-10-19

## Context

The roadmap runs to a first release of the whole design ([0222](0222-roadmap-to-first-release.md)), and every part of the design needs a step ([08-roadmap](../design/08-roadmap.md)). Several records fix parts of the order:

- The system library's entry table follows the IDL ([0154](0154-library-entry-table.md)).
- AArch64 comes with the first user-mode program ([0045](0045-aarch64-with-first-user-program.md)).
- The root task starts the scheduler, the memory service, the default pager and the loader, in that order ([0160](0160-root-task-starts-four.md)).
- User mode finds the CPUs ([0097](0097-user-mode-starts-cores.md)).
- The boot loader's check covers the ramdisk's list of packages ([0156](0156-boot-loader-checks-image.md)).
- The laptop needs USB storage ([0224](0224-laptop-usb-storage.md)).

## Options

1. **24 steps, as listed below.** Multicore follows the core services, and verified boot follows packages.
2. **Multicore last,** after the network step, as in the README's rough plan.
3. **Verified boot early,** right after the boot step, so every test image is signed from the start.

## Decision

Option 1. The steps, by phase:

**Phase 1. Foundations, on x86_64**

1. The repository, the build tool and its checks.
2. Booting the kernel: our fork of the boot loader, the image builder, the log, panics and the stack checker.
3. The kernel's base: its layout, entry and exit, guards, mitigations, firmware tables and timer.

**Phase 2. The kernel's objects, on x86_64**

4. Memory, capabilities and revoke.
5. Address spaces and threads.
6. System calls, the IDL compiler and the system library.

**Phase 3. User mode on both architectures**

7. The boot loader for AArch64, beside steps 3 to 6.
8. The first user-mode program, on x86_64 and AArch64.
9. IPC, faults and interrupts.

**Phase 4. The core services**

10. std's first stage, the page-table library and the root task.
11. The scheduler.
12. The memory service.
13. The default pager, the loader and the boot manifest.
14. The log, time and random services, and std's second stage.

**Phase 5. Multicore and devices**

15. Hardware discovery and multicore.
16. The device manager, the PCI bus driver and the IOMMU service.

**Phase 6. Storage, packages and verified boot**

17. NVMe, block storage and the filesystem.
18. USB and USB storage.
19. Packages, and std's third stage.
20. Verified boot and updates.

**Phase 7. Users, the network and the first release**

21. The console, login and the shell.
22. The network, and std's fourth stage.
23. Third-party programs.
24. The first release.

- [08-roadmap](../design/08-roadmap.md) gives each step's needs, what it builds, the decisions it implements, the topics it designs, its tests and its estimate.
- Each phase ends with a checkpoint ([0223](0223-qemu-and-laptop-checkpoints.md), [0249](0249-estimates-and-checkpoints.md)).

## Consequences

- **The IDL compiler comes before the system library,** so every interface has one source from the start ([0142](0142-own-idl.md), [0195](0195-bindings-safe-output.md)).
- **0045's phase is step 8** ([0254](0254-corrections.md)).
- **Concurrency is tested while most services are still young.** Multicore comes before drivers and storage, so AArch64's weaker memory ordering under load ([0009](0009-x86-64-first.md)) shows up early.
- **Images are signed from step 20 on.** Until then, the boot loader's check isn't built.
- **Steps 7 and 18 can run beside others,** as [0228](0228-step-start-and-done.md) allows.
- **0142's and 0150's questions on placing their work are answered,** and so are 05's and 06's on when their parts are built, together with [0229](0229-std-stages.md) and [0251](0251-later-work.md).
