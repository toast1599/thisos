# 08. Implementation roadmap, engineering practices and freeze

- **Day:** 8 (Mon 2026-10-19)
- **Status:** Frozen

## Covers

This is the exact plan for building thisOS. It sets the order in which every part of the design gets implemented, what each step delivers, how each step is tested, and when a step counts as done. It also settles the engineering practices that the whole build follows, from modularity and the repository layout to testing and CI, review, and the coding and documentation standards. This plan replaces the rough roadmap in the [README](../../README.md). The design and the roadmap froze together, with Day 8's commit ([0255](../decisions/0255-freeze.md)).

## Decided

Some decisions from earlier days already fix part of the order or the practices:

- [0006](../decisions/0006-non-rust-software.md) Non-Rust software is supported later; route still open (amended by [0082](../decisions/0082-corrections.md))
- [0008](../decisions/0008-architecture-neutral-design.md) Architecture-neutral design (amended by [0082](../decisions/0082-corrections.md))
- [0009](../decisions/0009-x86-64-first.md) x86_64 is the first architecture
- [0018](../decisions/0018-kernel-assurance.md) Kernel assurance: review, tests and fuzzing from the start; formal proofs later
- [0019](../decisions/0019-kernel-minimality.md) Kernel minimality: anything that can reasonably run in user mode does
- [0045](../decisions/0045-aarch64-with-first-user-program.md) AArch64 is brought up together with the first user-mode program (amended by [0254](../decisions/0254-corrections.md))
- [0046](../decisions/0046-scope-of-0008.md) Scope of 0008: the OS's own code, including platform-specific code (amended by [0152](../decisions/0152-kernel-log-and-reset.md) and [0213](../decisions/0213-power-capability.md))
- [0061](../decisions/0061-no-randomization-yet.md) Nothing is randomized at first, and the design keeps randomization possible
- [0068](../decisions/0068-no-kernel-heap.md) The kernel has no heap (amended by [0097](../decisions/0097-user-mode-starts-cores.md))
- [0077](../decisions/0077-swap-in-default-pager.md) Swap is a feature of the default pager; it comes later and is encrypted (amended by [0082](../decisions/0082-corrections.md))
- [0079](../decisions/0079-pointer-tagging-off.md) Hardware pointer tagging stays off for now
- [0148](../decisions/0148-freeze-covers-rules.md) The freeze covers the interface design, and each interface's version 1 is fixed when it's released

Day 7 ([07-engineering](07-engineering.md)) added these:

- [0188](../decisions/0188-upgrade-every-second-release.md) The toolchain is upgraded every second stable release
- [0191](../decisions/0191-audited-list-and-wrapper.md) A list of audited crates, enforced by a compiler wrapper, keeps unsafe code in the audited set
- [0193](../decisions/0193-vendored-reviewed-crates.md) Third-party crates in the OS's own code are vendored and reviewed (amended by [0243](../decisions/0243-audit-records-and-crate-reviews.md))
- [0194](../decisions/0194-reproducible-builds.md) The system image and the boot loader build reproducibly
- [0197](../decisions/0197-architecture-check.md) Crates are marked by architecture, and the build rejects architecture-specific code in generic crates (amended by [0232](../decisions/0232-traits-and-specific-crates.md))
- [0201](../decisions/0201-changes-after-freeze.md) After the freeze, a decision changes only through a new record that cites what building found
- [0202](../decisions/0202-offline-attackers.md) Attackers who get the machine while it's off are in scope
- [0209](../decisions/0209-no-rollback-protection.md) Older signed images still boot, and rollback protection is later work

Day 8 settled the scope, the machines and how open topics are handled:

- [0222](../decisions/0222-roadmap-to-first-release.md) The roadmap runs from an empty repository to a first release of the whole design
- [0223](../decisions/0223-qemu-and-laptop-checkpoints.md) thisOS is built and tested in QEMU, with a checkpoint on the development laptop at the end of each phase
- [0224](../decisions/0224-laptop-usb-storage.md) On the laptop, thisOS keeps its storage on a USB disk, and never opens the internal disk
- [0225](../decisions/0225-framebuffer-log-at-failures.md) The kernel shows its log on the framebuffer when it fails, and while bring-up images start
- [0226](../decisions/0226-open-topics-at-their-step.md) Topics no record decides are designed at the step that needs them

Then the steps themselves:

- [0227](../decisions/0227-steps-and-phases.md) The roadmap has 24 steps in 7 phases, each after the steps it needs
- [0228](../decisions/0228-step-start-and-done.md) A step starts once the steps it needs are done, and is done when it's tested, reviewed and mapped
- [0229](../decisions/0229-std-stages.md) std comes in four stages, each done when upstream's tests for it pass
- [0230](../decisions/0230-interfaces-released-with-sdk.md) Interfaces that third-party programs reach are released with the SDK

On modularity:

- [0231](../decisions/0231-repository-layout.md) One repository holds thisOS, with a Cargo workspace for each kind of target
- [0232](../decisions/0232-traits-and-specific-crates.md) Generic crates state their hardware needs as traits, and specific crates implement them
- [0233](../decisions/0233-simulated-machine.md) The kernel's generic part also runs on the host, on a simulated machine
- [0234](../decisions/0234-tests-cite-records.md) Tests name the records they check, and the build tool maps every decision to its tests

For testing and CI:

- [0235](../decisions/0235-kinds-of-tests.md) Every change runs host, QEMU, check and attack tests, and Miri runs on a test-only nightly
- [0236](../decisions/0236-program-output-second-serial.md) Programs write to a second serial port until the log service takes it, and test runs end through the power capability
- [0237](../decisions/0237-public-repository-and-ci.md) thisOS is public on GitHub, where CI runs the build tool's `ci` command for every change
- [0238](../decisions/0238-pushes-and-pull-requests.md) Maintainers push to main after a local `ci` run, and others send pull requests
- [0239](../decisions/0239-security-reports.md) Security problems are reported privately, through GitHub
- [0240](../decisions/0240-timing-on-laptop.md) Benchmarks and time budgets run on the development laptop
- [0241](../decisions/0241-reproducibility-checks.md) The image and the boot loader are built twice in CI, and rebuilt elsewhere before signing

Review and kernel assurance:

- [0242](../decisions/0242-review-tiers.md) Changes are reviewed in three tiers, by role
- [0243](../decisions/0243-audit-records-and-crate-reviews.md) Each audited crate has a record beside the list, and cargo-vet records crate reviews
- [0244](../decisions/0244-imported-crate-reviews.md) Crate reviews may be imported from named organizations, outside the audited set
- [0245](../decisions/0245-coverage-and-fuzzing.md) Host tests cover every line of the generic kernel, and system calls are fuzzed from step 4
- [0246](../decisions/0246-interruption-stretch-limit.md) Work between interruption points is limited in code, and no stretch may take more than 20 µs
- [0247](../decisions/0247-kernel-size-ratchet.md) A record rule and a size ratchet keep the kernel small

The standards, the estimates, changes to the plan and later work:

- [0248](../decisions/0248-coding-and-doc-standards.md) The code follows one set of coding and documentation standards
- [0249](../decisions/0249-estimates-and-checkpoints.md) Steps are estimated by the code they add, and calibrated at each phase's checkpoint
- [0250](../decisions/0250-plan-changes-after-freeze.md) After the freeze, steps change only through a record that cites what building found
- [0251](../decisions/0251-later-work.md) Later work stays unscheduled, and the first release states the gaps it leaves

Placing the steps turned up gaps, and a slip in an earlier record:

- [0252](../decisions/0252-time-service-starts-early.md) The time service starts early, and serves wall-clock time once the clock driver reports it
- [0253](../decisions/0253-ramdisk-log-and-clocks.md) The ramdisk also holds the log service and the clock drivers
- [0254](../decisions/0254-corrections.md) Corrections to 0045

And the freeze itself:

- [0255](../decisions/0255-freeze.md) The design and the roadmap freeze with Day 8's commit, on 2026-10-19

## The roadmap

The roadmap starts from an empty repository and ends at a first release of the whole design ([0222](../decisions/0222-roadmap-to-first-release.md)). Everything that isn't on the list of later work gets built, and tested on both architectures. What the first release publishes is a signed system image and boot loader, the SDK, and version 1 of the interfaces that third-party programs reach ([0230](../decisions/0230-interfaces-released-with-sdk.md)).

We build and test every step in QEMU. x86_64 runs on q35 with OVMF under KVM, AArch64 runs on virt with AAVMF under emulation, and each runs both with and without an IOMMU ([0223](../decisions/0223-qemu-and-laptop-checkpoints.md)). The development laptop comes in at the end of each phase: it boots that phase's image from a USB stick and runs it as far as its devices allow.

The work splits into 24 steps, in 7 phases ([0227](../decisions/0227-steps-and-phases.md)). A step starts once the steps it needs are done ([0228](../decisions/0228-step-start-and-done.md)), and it counts as done when all of this holds:

- its tests pass on every architecture it supports, which means x86_64 for steps 1 to 6, AArch64 for step 7, and both from step 8 on;
- CI passes ([0237](../decisions/0237-public-repository-and-ci.md));
- its review is finished ([0242](../decisions/0242-review-tiers.md));
- every decision it implements has its tests in the coverage map ([0234](../decisions/0234-tests-cite-records.md));
- its documents are up to date, and the table below marks it done.

| # | Step | Needs | Status |
|---|---|---|---|
| 1 | The repository, the build tool and its checks | — | Not started |
| 2 | Booting the kernel | 1 | Not started |
| 3 | The kernel's base | 2 | Not started |
| 4 | Memory, capabilities and revoke | 3 | Not started |
| 5 | Address spaces and threads | 4 | Not started |
| 6 | System calls, the IDL compiler and the system library | 5 | Not started |
| 7 | The boot loader for AArch64 | 2 | Not started |
| 8 | The first user-mode program, on x86_64 and AArch64 | 6, 7 | Not started |
| 9 | IPC, faults and interrupts | 8 | Not started |
| 10 | std's first stage, the page-table library and the root task | 9 | Not started |
| 11 | The scheduler | 10 | Not started |
| 12 | The memory service | 11 | Not started |
| 13 | The default pager, the loader and the boot manifest | 12 | Not started |
| 14 | The log, time and random services, and std's second stage | 13 | Not started |
| 15 | Hardware discovery and multicore | 14 | Not started |
| 16 | The device manager, the PCI bus driver and the IOMMU service | 15 | Not started |
| 17 | NVMe, block storage and the filesystem | 16 | Not started |
| 18 | USB and USB storage | 16 | Not started |
| 19 | Packages, and std's third stage | 17 | Not started |
| 20 | Verified boot and updates | 19 | Not started |
| 21 | The console, login and the shell | 18, 19 | Not started |
| 22 | The network, and std's fourth stage | 19 | Not started |
| 23 | Third-party programs | 20, 21, 22 | Not started |
| 24 | The first release | 23 | Not started |

## The steps

For each step below you'll find the steps it needs, what it builds, the decisions it implements, the open topics it designs, and the tests it adds on top of the standard kinds ([0235](../decisions/0235-kinds-of-tests.md)). A checkpoint closes each phase ([0249](../decisions/0249-estimates-and-checkpoints.md)). The estimates have a section of their own, under [Estimates](#estimates).

### Phase 1. Foundations, on x86_64

#### 1. The repository, the build tool and its checks

Nothing comes before this step.

It lays down the repository's workspaces ([0231](../decisions/0231-repository-layout.md)) and the pinned toolchain ([0187](../decisions/0187-pinned-stable-toolchain.md)). It also brings the build tool ([0190](../decisions/0190-cargo-and-build-tool.md)), along with the compiler wrapper ([0191](../decisions/0191-audited-list-and-wrapper.md), [0192](../decisions/0192-build-scripts-reviewed.md)), the architecture and dependency checks ([0197](../decisions/0197-architecture-check.md), [0232](../decisions/0232-traits-and-specific-crates.md)), the offline sandbox ([0192](../decisions/0192-build-scripts-reviewed.md)), the count of the kernel's size ([0247](../decisions/0247-kernel-size-ratchet.md)) and the map of records to tests ([0234](../decisions/0234-tests-cite-records.md)). Vendoring is set up too, with cargo-vet, imported reviews and cargo-deny ([0193](../decisions/0193-vendored-reviewed-crates.md), [0243](../decisions/0243-audit-records-and-crate-reviews.md), [0244](../decisions/0244-imported-crate-reviews.md)). And the project gets the `ci` command and GitHub's workflows ([0237](../decisions/0237-public-repository-and-ci.md)), the ruleset on main ([0238](../decisions/0238-pushes-and-pull-requests.md)), and `SECURITY.md` ([0239](../decisions/0239-security-reports.md)).

- **Implements:** [0008](../decisions/0008-architecture-neutral-design.md), [0010](../decisions/0010-64-bit-only.md), [0017](../decisions/0017-unsafe-code-kernel-and-services.md), [0018](../decisions/0018-kernel-assurance.md), [0019](../decisions/0019-kernel-minimality.md), [0037](../decisions/0037-kernel-toolchain.md), [0044](../decisions/0044-system-image-is-os-shipped.md), [0046](../decisions/0046-scope-of-0008.md), [0187](../decisions/0187-pinned-stable-toolchain.md), [0190](../decisions/0190-cargo-and-build-tool.md), [0191](../decisions/0191-audited-list-and-wrapper.md), [0192](../decisions/0192-build-scripts-reviewed.md), [0193](../decisions/0193-vendored-reviewed-crates.md), [0197](../decisions/0197-architecture-check.md), [0199](../decisions/0199-license.md), [0200](../decisions/0200-contributions.md), [0231](../decisions/0231-repository-layout.md), [0232](../decisions/0232-traits-and-specific-crates.md), [0234](../decisions/0234-tests-cite-records.md), [0237](../decisions/0237-public-repository-and-ci.md), [0238](../decisions/0238-pushes-and-pull-requests.md), [0239](../decisions/0239-security-reports.md), [0243](../decisions/0243-audit-records-and-crate-reviews.md), [0244](../decisions/0244-imported-crate-reviews.md), [0247](../decisions/0247-kernel-size-ratchet.md), [0248](../decisions/0248-coding-and-doc-standards.md).
- **Designs:** how tests name the records they check ([0234](../decisions/0234-tests-cite-records.md)), the project's set of lints ([0248](../decisions/0248-coding-and-doc-standards.md)), and the CI workflows and the nightly runs ([0237](../decisions/0237-public-repository-and-ci.md)).
- **Tests:** each check rejects planted violations. We plant unsafe code or a feature gate in a crate that isn't on the list, a build script that links native code, architecture code in a generic crate, a generic crate that depends on a specific one, a crate with a known advisory or a refused license, and a crate with no review.

#### 2. Booting the kernel

Step 2 needs step 1.

Our fork of the `bootloader` crate is vendored here, and the build tool builds its UEFI stage ([0149](../decisions/0149-bootloader-crate-uefi.md), [0190](../decisions/0190-cargo-and-build-tool.md)). The image builder makes a GPT disk image with a FAT boot partition, and a ramdisk whose header carries the boot settings and the image's version ([0153](../decisions/0153-build-unpacks-ramdisk.md), [0157](../decisions/0157-boot-settings-in-ramdisk.md), [0194](../decisions/0194-reproducible-builds.md), [0209](../decisions/0209-no-rollback-protection.md)). The kernel crates are built for `x86_64-unknown-none` with `core` from source ([0189](../decisions/0189-kernel-none-targets.md)), and they start from the boot information ([0151](../decisions/0151-boot-info-from-crate.md)).

The kernel's log lives in its ring buffer and goes out on COM1 ([0152](../decisions/0152-kernel-log-and-reset.md)). It's also drawn on the framebuffer at failures and in bring-up images ([0225](../decisions/0225-framebuffer-log-at-failures.md)). Panics halt or reboot, by the boot setting ([0110](../decisions/0110-kernel-panic.md)), and the reset comes in as well ([0152](../decisions/0152-kernel-log-and-reset.md)). Rounding the step out are the stack checker ([0198](../decisions/0198-stack-depth-checker.md)), the QEMU runner, the kernel's test build and the reproducibility check ([0241](../decisions/0241-reproducibility-checks.md)).

- **Implements:** [0009](../decisions/0009-x86-64-first.md), [0060](../decisions/0060-kernel-stack-per-core.md), [0068](../decisions/0068-no-kernel-heap.md), [0110](../decisions/0110-kernel-panic.md), [0149](../decisions/0149-bootloader-crate-uefi.md), [0151](../decisions/0151-boot-info-from-crate.md), [0152](../decisions/0152-kernel-log-and-reset.md), [0153](../decisions/0153-build-unpacks-ramdisk.md), [0157](../decisions/0157-boot-settings-in-ramdisk.md), [0189](../decisions/0189-kernel-none-targets.md), [0190](../decisions/0190-cargo-and-build-tool.md), [0194](../decisions/0194-reproducible-builds.md), [0198](../decisions/0198-stack-depth-checker.md), [0209](../decisions/0209-no-rollback-protection.md), [0225](../decisions/0225-framebuffer-log-at-failures.md), [0241](../decisions/0241-reproducibility-checks.md).
- **Designs:** the image's version numbers ([0209](../decisions/0209-no-rollback-protection.md)).
- **Tests:** the log reaches COM1 and the ring buffer. A forced panic halts or reboots by the setting and draws its log on the framebuffer, while a bring-up image draws its log as it's written. The stack checker rejects planted recursion, and an indirect call it can't resolve. Finally, two varied builds come out as the same bytes.

#### 3. The kernel's base

This step needs step 2, and gives the kernel its base.

The kernel takes the upper-half layout, with the widest addressing the CPU offers ([0063](../decisions/0063-widest-addressing.md), [0066](../decisions/0066-kernel-upper-half.md), [0073](../decisions/0073-kernel-layout-in-implementation.md)). It maps its own memory only ([0047](../decisions/0047-kernel-mapping.md), [0055](../decisions/0055-kernel-maps-only-objects.md)), and sets up the object window's region and its tables ([0067](../decisions/0067-object-window-offset.md)). The boot core gets its stack, with guard pages and fault stacks ([0060](../decisions/0060-kernel-stack-per-core.md)). Exceptions and interrupts get their entry and exit, and interrupts stay off between interruption points ([0089](../decisions/0089-interrupts-off-in-kernel.md)).

On the protection side there are the guards against touching or running program memory ([0075](../decisions/0075-user-guards-always-on.md)), and the detection of CPU flaws with their mitigations ([0100](../decisions/0100-cpu-flaw-mitigations.md)). The kernel reads ACPI's MADT and FADT ([0151](../decisions/0151-boot-info-from-crate.md), [0152](../decisions/0152-kernel-log-and-reset.md), [0213](../decisions/0213-power-capability.md)), and drives the local APIC and its timer deadline ([0094](../decisions/0094-one-deadline-per-core.md)). The step also builds the counter's access control ([0113](../decisions/0113-counter-capability.md)) and the kernel lock ([0095](../decisions/0095-one-kernel-lock.md)).

It implements [0032](../decisions/0032-minimal-hardware-discovery.md), [0047](../decisions/0047-kernel-mapping.md), [0055](../decisions/0055-kernel-maps-only-objects.md), [0060](../decisions/0060-kernel-stack-per-core.md), [0063](../decisions/0063-widest-addressing.md), [0066](../decisions/0066-kernel-upper-half.md), [0067](../decisions/0067-object-window-offset.md), [0073](../decisions/0073-kernel-layout-in-implementation.md), [0075](../decisions/0075-user-guards-always-on.md), [0079](../decisions/0079-pointer-tagging-off.md), [0089](../decisions/0089-interrupts-off-in-kernel.md), [0094](../decisions/0094-one-deadline-per-core.md), [0095](../decisions/0095-one-kernel-lock.md), [0100](../decisions/0100-cpu-flaw-mitigations.md), [0113](../decisions/0113-counter-capability.md), [0151](../decisions/0151-boot-info-from-crate.md), [0152](../decisions/0152-kernel-log-and-reset.md), [0213](../decisions/0213-power-capability.md).
The tests plant accesses, and the guards fault on them. QEMU's CPU models show that mitigations are chosen per CPU model, and a test build forces the unmapped kernel ([0047](../decisions/0047-kernel-mapping.md)). Paging works with 4 and with 5 levels. A fault on a guard page lands on a fault stack, and pointer tagging stays off ([0079](../decisions/0079-pointer-tagging-off.md)).

Checkpoint 1 ends the phase. The laptop boots the image from a USB stick, shows the bring-up log, and draws the log of a forced panic ([0225](../decisions/0225-framebuffer-log-at-failures.md)).

### Phase 2. The kernel's objects, on x86_64

#### 4. Memory, capabilities and revoke

Needs step 3. This is where memory capabilities appear, along with making kernel objects from them ([0025](../decisions/0025-user-mode-memory.md), [0026](../decisions/0026-programs-pay-kernel-objects.md)), and memory is cleared before it's reused ([0051](../decisions/0051-kernel-clears-memory.md)). Capability spaces come in ([0126](../decisions/0126-capability-space-radix-table.md), [0127](../decisions/0127-capability-space-per-program.md)), with rights and badges ([0128](../decisions/0128-rights-and-badges.md)), and so do the records of what was made from what, and revoke ([0053](../decisions/0053-revoke-destroys-derived.md), [0129](../decisions/0129-revoke-any-capability.md)).

The kernel gets its interruption points, with their work limits and their yields of the lock ([0089](../decisions/0089-interrupts-off-in-kernel.md), [0218](../decisions/0218-lock-yield.md), [0246](../decisions/0246-interruption-stretch-limit.md)), and the object window gets lazy unmapping and batched flushes ([0096](../decisions/0096-object-window-lazy-unmap.md), [0217](../decisions/0217-window-flush-batched.md)). The step also builds the simulated machine ([0233](../decisions/0233-simulated-machine.md)), and fuzzes system calls on it ([0245](../decisions/0245-coverage-and-fuzzing.md)).

- **Implements:** [0018](../decisions/0018-kernel-assurance.md), [0020](../decisions/0020-capabilities.md), [0025](../decisions/0025-user-mode-memory.md), [0026](../decisions/0026-programs-pay-kernel-objects.md), [0051](../decisions/0051-kernel-clears-memory.md), [0053](../decisions/0053-revoke-destroys-derived.md), [0055](../decisions/0055-kernel-maps-only-objects.md), [0067](../decisions/0067-object-window-offset.md), [0089](../decisions/0089-interrupts-off-in-kernel.md), [0096](../decisions/0096-object-window-lazy-unmap.md), [0126](../decisions/0126-capability-space-radix-table.md), [0127](../decisions/0127-capability-space-per-program.md), [0128](../decisions/0128-rights-and-badges.md), [0129](../decisions/0129-revoke-any-capability.md), [0217](../decisions/0217-window-flush-batched.md), [0218](../decisions/0218-lock-yield.md), [0233](../decisions/0233-simulated-machine.md), [0245](../decisions/0245-coverage-and-fuzzing.md), [0246](../decisions/0246-interruption-stretch-limit.md).
- **Tests:** they cover every line of the generic kernel so far ([0245](../decisions/0245-coverage-and-fuzzing.md)), revoke over deep trees of derived capabilities, memory cleared before each reuse, and the simulator's work counts ([0246](../decisions/0246-interruption-stretch-limit.md)). The attack tests try window-flush loops ([0217](../decisions/0217-window-flush-batched.md)) and revokes that would hold the lock ([0218](../decisions/0218-lock-yield.md)).

#### 5. Address spaces and threads

Step 5 needs step 4. It builds address spaces ([0083](../decisions/0083-program-is-address-space.md)), gated by the capability for creating programs ([0031](../decisions/0031-user-mode-loader.md)), and page tables that user mode links ([0052](../decisions/0052-user-mode-page-tables.md)). Those support every page size ([0069](../decisions/0069-all-page-sizes.md)), and on x86_64 the kernel's half is copied into them ([0066](../decisions/0066-kernel-upper-half.md)). Two mapping rules apply: the execute right, and no memory writable and executable at once ([0043](../decisions/0043-executable-memory-capability.md), [0054](../decisions/0054-kernel-enforces-execute.md)). Device memory and I/O ports are added too ([0168](../decisions/0168-device-memory-and-ports.md)).

Then threads. Their floating-point state is saved on every switch ([0101](../decisions/0101-eager-fpu-state.md)), and each has the thread pointer ([0177](../decisions/0177-locks-on-parking.md)). Priorities arrive with priority capabilities ([0033](../decisions/0033-cpu-fairness.md), [0087](../decisions/0087-thread-priorities.md), [0090](../decisions/0090-priorities-in-kernel.md)), along with the per-core deadline ([0094](../decisions/0094-one-deadline-per-core.md)) and the scheduler thread's place above every priority ([0162](../decisions/0162-scheduler-above-priorities.md)). The kernel keeps running totals of CPU time ([0134](../decisions/0134-scheduler-interface.md)). On switches it flushes the predictors ([0080](../decisions/0080-predictor-flush-both-ways.md), [0136](../decisions/0136-predictor-flush-flag.md)) and switches the counter's access with the program ([0113](../decisions/0113-counter-capability.md)).

- **Implements:** [0031](../decisions/0031-user-mode-loader.md), [0033](../decisions/0033-cpu-fairness.md), [0043](../decisions/0043-executable-memory-capability.md), [0052](../decisions/0052-user-mode-page-tables.md), [0054](../decisions/0054-kernel-enforces-execute.md), [0066](../decisions/0066-kernel-upper-half.md), [0069](../decisions/0069-all-page-sizes.md), [0080](../decisions/0080-predictor-flush-both-ways.md), [0083](../decisions/0083-program-is-address-space.md), [0087](../decisions/0087-thread-priorities.md), [0090](../decisions/0090-priorities-in-kernel.md), [0094](../decisions/0094-one-deadline-per-core.md), [0101](../decisions/0101-eager-fpu-state.md), [0113](../decisions/0113-counter-capability.md), [0134](../decisions/0134-scheduler-interface.md), [0136](../decisions/0136-predictor-flush-flag.md), [0162](../decisions/0162-scheduler-above-priorities.md), [0168](../decisions/0168-device-memory-and-ports.md), [0177](../decisions/0177-locks-on-parking.md).
- **Tests:** memory that is mapped writable anywhere can't be mapped executable anywhere. Flushes happen exactly when asked for, and floating-point state never crosses between threads.

#### 6. System calls, the IDL compiler and the system library

This step needs step 5. System calls get their entry and exit, with the `sysret` guard ([0125](../decisions/0125-sysret-guard.md)); the kernel accepts them only from the system library's code ([0117](../decisions/0117-system-library.md)), and they carry data only in registers ([0072](../decisions/0072-registers-only.md)).

Next comes the IDL compiler, with the fixed-layout types, the entry table and the reference documentation ([0142](../decisions/0142-own-idl.md), [0195](../decisions/0195-bindings-safe-output.md)), and the compatibility checker ([0143](../decisions/0143-add-only-versioning.md), [0196](../decisions/0196-released-interface-copies.md)). The system library follows, with its entry table ([0117](../decisions/0117-system-library.md), [0144](../decisions/0144-c-calls-and-what-crosses.md), [0154](../decisions/0154-library-entry-table.md)) and the joining of split calls ([0124](../decisions/0124-large-calls-split.md)). The first part of the IDL runtime is built here ([0195](../decisions/0195-bindings-safe-output.md)), and the startup data and the boot information are defined as IDL types ([0146](../decisions/0146-startup-data-block.md), [0159](../decisions/0159-root-task-boot-information.md)).

- **Implements:** [0021](../decisions/0021-ipc-simple-upgradable.md), [0072](../decisions/0072-registers-only.md), [0117](../decisions/0117-system-library.md), [0124](../decisions/0124-large-calls-split.md), [0125](../decisions/0125-sysret-guard.md), [0142](../decisions/0142-own-idl.md), [0143](../decisions/0143-add-only-versioning.md), [0144](../decisions/0144-c-calls-and-what-crosses.md), [0146](../decisions/0146-startup-data-block.md), [0154](../decisions/0154-library-entry-table.md), [0159](../decisions/0159-root-task-boot-information.md), [0195](../decisions/0195-bindings-safe-output.md), [0196](../decisions/0196-released-interface-copies.md).
- **Interfaces:** the system library's, the startup data's and the boot information's. They're defined here but not released ([0230](../decisions/0230-interfaces-released-with-sdk.md)).
- **Tests:** a return address outside the program half never reaches `sysret` ([0125](../decisions/0125-sysret-guard.md)), and system calls from outside the library are refused ([0117](../decisions/0117-system-library.md)). Against test copies, the checker rejects planted incompatible changes ([0196](../decisions/0196-released-interface-copies.md)).

Checkpoint 2: the laptop runs the kernel's test build, and the results appear on the framebuffer ([0225](../decisions/0225-framebuffer-log-at-failures.md)).

### Phase 3. User mode on both architectures

#### 7. The boot loader for AArch64

This step needs step 2. Our fork gets AArch64 support through UEFI, with the device tree's address in the boot information ([0150](../decisions/0150-bootloader-aarch64.md)), and QEMU for AArch64 and AAVMF get installed on the development laptop ([0009](../decisions/0009-x86-64-first.md), [0011](../decisions/0011-aarch64-second.md)).

- **Implements:** [0009](../decisions/0009-x86-64-first.md), [0011](../decisions/0011-aarch64-second.md), [0150](../decisions/0150-bootloader-aarch64.md).
- **Tests:** the fork starts a test kernel on virt under AAVMF.

#### 8. The first user-mode program, on x86_64 and AArch64

Needs steps 6 and 7. First the kernel's AArch64 parts: exception vectors, the MMU with the 4 KiB granule ([0069](../decisions/0069-all-page-sizes.md)), PAN and execute-never ([0075](../decisions/0075-user-guards-always-on.md)), floating-point and vector state ([0101](../decisions/0101-eager-fpu-state.md)), the generic timer, GICv3, PL011, PSCI and the device tree ([0151](../decisions/0151-boot-info-from-crate.md), [0152](../decisions/0152-kernel-log-and-reset.md)), and mitigations ([0100](../decisions/0100-cpu-flaw-mitigations.md)).

Then the root task. Its first address space comes from the ramdisk's header ([0052](../decisions/0052-user-mode-page-tables.md), [0153](../decisions/0153-build-unpacks-ramdisk.md)), and it gets its startup data, boot information and initial capabilities, the power capability among them ([0024](../decisions/0024-root-task.md), [0159](../decisions/0159-root-task-boot-information.md), [0213](../decisions/0213-power-capability.md)). It runs alone ([0161](../decisions/0161-root-task-before-scheduler.md)), and a fault in it is handled like a panic ([0115](../decisions/0115-root-task-fault-boot-setting.md)). For now it's a stub that writes to the second serial port and ends through the power capability ([0236](../decisions/0236-program-output-second-serial.md)). The system library and the stack checker also come up on AArch64, and CI runs on both architectures ([0228](../decisions/0228-step-start-and-done.md)).

- **Implements:** [0011](../decisions/0011-aarch64-second.md), [0024](../decisions/0024-root-task.md), [0032](../decisions/0032-minimal-hardware-discovery.md), [0045](../decisions/0045-aarch64-with-first-user-program.md), [0052](../decisions/0052-user-mode-page-tables.md), [0069](../decisions/0069-all-page-sizes.md), [0075](../decisions/0075-user-guards-always-on.md), [0079](../decisions/0079-pointer-tagging-off.md), [0100](../decisions/0100-cpu-flaw-mitigations.md), [0101](../decisions/0101-eager-fpu-state.md), [0115](../decisions/0115-root-task-fault-boot-setting.md), [0117](../decisions/0117-system-library.md), [0151](../decisions/0151-boot-info-from-crate.md), [0153](../decisions/0153-build-unpacks-ramdisk.md), [0159](../decisions/0159-root-task-boot-information.md), [0161](../decisions/0161-root-task-before-scheduler.md), [0213](../decisions/0213-power-capability.md), [0236](../decisions/0236-program-output-second-serial.md).
- **Designs:** how the root task finds the second serial port before hardware discovery runs ([0236](../decisions/0236-program-output-second-serial.md)).
- **Tests:** the stub's output arrives on both architectures. The power capability reboots the machine, powers it off and fails it, and the root task's initial capabilities match its boot information.

#### 9. IPC, faults and interrupts

Step 9 needs step 8. It starts with IPC itself: endpoints, reply objects and notifications ([0118](../decisions/0118-calls-and-notifications.md), [0120](../decisions/0120-reply-objects.md)), messages of 8 words ([0119](../decisions/0119-message-eight-words.md)), capabilities in messages ([0123](../decisions/0123-capabilities-in-messages.md)), and bound notifications ([0122](../decisions/0122-bound-notifications.md)). Calls switch directly ([0091](../decisions/0091-direct-switch-on-calls.md)), callers lend to busy services ([0121](../decisions/0121-callers-help-busy-services.md)), endpoints serve by priority ([0220](../decisions/0220-endpoint-priority-order.md)), and a wakeup enters the scheduler only when the woken thread may run first ([0219](../decisions/0219-wakeup-preemption.md)). The kernel also reports to the scheduler thread, and waits can be cancelled ([0092](../decisions/0092-scheduler-keeps-time.md), [0134](../decisions/0134-scheduler-interface.md)).

Faults are the second half. Page faults go to pagers ([0081](../decisions/0081-pager-per-thread.md)) and exceptions to handlers ([0102](../decisions/0102-exception-handler.md)); pagers can decline ([0106](../decisions/0106-pager-passes-on-faults.md)), and fault messages carry the recovery register ([0133](../decisions/0133-recovery-point-in-register.md)). A suspended thread's address space gets a notification ([0103](../decisions/0103-fault-without-handler.md), [0132](../decisions/0132-events-notify-then-call.md)), and handlers can be changed ([0130](../decisions/0130-handlers-changeable.md)). Interrupts become notifications, from lines and from MSIs, on the APIC and the GIC ([0167](../decisions/0167-interrupts-as-notifications.md)). The step also adds the boot core's object ([0165](../decisions/0165-core-objects.md)) and runs the system-call fuzzer in QEMU ([0245](../decisions/0245-coverage-and-fuzzing.md)).

- **Implements:** [0018](../decisions/0018-kernel-assurance.md), [0019](../decisions/0019-kernel-minimality.md), [0021](../decisions/0021-ipc-simple-upgradable.md), [0027](../decisions/0027-user-mode-pagers.md), [0036](../decisions/0036-ipc-message-max.md), [0058](../decisions/0058-shared-and-handed-over.md), [0072](../decisions/0072-registers-only.md), [0081](../decisions/0081-pager-per-thread.md), [0085](../decisions/0085-clients-pay-cpu-time.md), [0091](../decisions/0091-direct-switch-on-calls.md), [0092](../decisions/0092-scheduler-keeps-time.md), [0102](../decisions/0102-exception-handler.md), [0103](../decisions/0103-fault-without-handler.md), [0106](../decisions/0106-pager-passes-on-faults.md), [0118](../decisions/0118-calls-and-notifications.md), [0119](../decisions/0119-message-eight-words.md), [0120](../decisions/0120-reply-objects.md), [0121](../decisions/0121-callers-help-busy-services.md), [0122](../decisions/0122-bound-notifications.md), [0123](../decisions/0123-capabilities-in-messages.md), [0124](../decisions/0124-large-calls-split.md), [0130](../decisions/0130-handlers-changeable.md), [0132](../decisions/0132-events-notify-then-call.md), [0133](../decisions/0133-recovery-point-in-register.md), [0134](../decisions/0134-scheduler-interface.md), [0165](../decisions/0165-core-objects.md), [0167](../decisions/0167-interrupts-as-notifications.md), [0219](../decisions/0219-wakeup-preemption.md), [0220](../decisions/0220-endpoint-priority-order.md), [0245](../decisions/0245-coverage-and-fuzzing.md), [0247](../decisions/0247-kernel-size-ratchet.md).
- **Tests:** fuzzing on both architectures, plus attacks: inversion through a busy service ([0121](../decisions/0121-callers-help-busy-services.md)), and wakeup storms on a busy core ([0219](../decisions/0219-wakeup-preemption.md)). Once the step is done, the kernel's size counts become its budget ([0247](../decisions/0247-kernel-size-ratchet.md)).

Checkpoint 3: the laptop runs the stub's tests. It powers off when they pass and shows the failure on the framebuffer when one fails ([0225](../decisions/0225-framebuffer-log-at-failures.md)).

### Phase 4. The core services

#### 10. std's first stage, the page-table library and the root task

Needs step 9. The target specs and std's patch come first ([0175](../decisions/0175-rust-target-names.md), [0187](../decisions/0187-pinned-stable-toolchain.md)), together with std's first stage ([0229](../decisions/0229-std-stages.md)), and then the page-table library with its architecture crates ([0052](../decisions/0052-user-mode-page-tables.md)).

The root task is the big piece. It reads its boot information ([0159](../decisions/0159-root-task-boot-information.md)), keeps back its memory pool ([0164](../decisions/0164-root-task-memory-pool.md)), and builds and pages the programs it starts, as it will the four services ([0057](../decisions/0057-pager-fault-chain.md), [0071](../decisions/0071-three-memory-programs.md), [0160](../decisions/0160-root-task-starts-four.md)). It supervises them ([0104](../decisions/0104-supervisor.md), [0105](../decisions/0105-supervisor-reclaims.md)), and when an essential service fails it fails the machine ([0216](../decisions/0216-essential-service-failure.md)). The pager's protocol is written in the IDL.

- **Implements:** [0003](../decisions/0003-rust-std-no-libc.md), [0004](../decisions/0004-not-unix.md), [0052](../decisions/0052-user-mode-page-tables.md), [0057](../decisions/0057-pager-fault-chain.md), [0071](../decisions/0071-three-memory-programs.md), [0104](../decisions/0104-supervisor.md), [0105](../decisions/0105-supervisor-reclaims.md), [0109](../decisions/0109-panic-strategy.md), [0145](../decisions/0145-std-over-native-api.md), [0153](../decisions/0153-build-unpacks-ramdisk.md), [0159](../decisions/0159-root-task-boot-information.md), [0160](../decisions/0160-root-task-starts-four.md), [0164](../decisions/0164-root-task-memory-pool.md), [0175](../decisions/0175-rust-target-names.md), [0182](../decisions/0182-all-of-std.md), [0187](../decisions/0187-pinned-stable-toolchain.md), [0216](../decisions/0216-essential-service-failure.md), [0229](../decisions/0229-std-stages.md).
- **Tests:** upstream's std tests for stage 1, and a check that an essential service which fails takes the machine down by the boot setting ([0216](../decisions/0216-essential-service-failure.md)).

#### 11. The scheduler

Needs step 10. This step builds the scheduler ([0088](../decisions/0088-user-mode-scheduler.md)), which has a thread on each core ([0093](../decisions/0093-scheduler-per-core.md)). It gives fair shares by account within each priority ([0033](../decisions/0033-cpu-fairness.md), [0084](../decisions/0084-shares-belong-to-accounts.md), [0086](../decisions/0086-priorities-per-program-shares-per-account.md)), handles time slices, and does sleep and timeouts through the deadline ([0094](../decisions/0094-one-deadline-per-core.md)). It also takes the kernel's reports ([0092](../decisions/0092-scheduler-keeps-time.md), [0134](../decisions/0134-scheduler-interface.md)).

The scheduler runs above every priority ([0114](../decisions/0114-scheduler-priority.md), [0162](../decisions/0162-scheduler-above-priorities.md)) and starts before any other program ([0161](../decisions/0161-root-task-before-scheduler.md)). Around it sit core-sharing groups ([0135](../decisions/0135-core-sharing-from-scheduler.md)), CPU accounts ([0140](../decisions/0140-accounts-capability-pair.md)) and reports of CPU use ([0111](../decisions/0111-resource-accounting.md)). Its first capabilities go to the root task ([0163](../decisions/0163-services-hand-first-capabilities.md)).

- **Implements:** [0023](../decisions/0023-no-quotas.md), [0033](../decisions/0033-cpu-fairness.md), [0084](../decisions/0084-shares-belong-to-accounts.md), [0085](../decisions/0085-clients-pay-cpu-time.md), [0086](../decisions/0086-priorities-per-program-shares-per-account.md), [0088](../decisions/0088-user-mode-scheduler.md), [0092](../decisions/0092-scheduler-keeps-time.md), [0093](../decisions/0093-scheduler-per-core.md), [0094](../decisions/0094-one-deadline-per-core.md), [0111](../decisions/0111-resource-accounting.md), [0114](../decisions/0114-scheduler-priority.md), [0134](../decisions/0134-scheduler-interface.md), [0135](../decisions/0135-core-sharing-from-scheduler.md), [0140](../decisions/0140-accounts-capability-pair.md), [0161](../decisions/0161-root-task-before-scheduler.md), [0162](../decisions/0162-scheduler-above-priorities.md), [0163](../decisions/0163-services-hand-first-capabilities.md).
- **Interfaces:** the scheduler's protocol.
- **Tests:** accounts at one priority share time evenly, and no program raises its own priority. A woken thread below the running one doesn't enter the scheduler ([0219](../decisions/0219-wakeup-preemption.md)). Nothing caps CPU time while the CPU is free ([0023](../decisions/0023-no-quotas.md)).

#### 12. The memory service

Needs step 11. The memory service reserves memory when it grants it ([0048](../decisions/0048-reserve-on-grant.md)) and names payers by badge ([0064](../decisions/0064-memory-belongs-to-payer.md), [0137](../decisions/0137-memory-capability-per-program.md)). It has memory accounts ([0084](../decisions/0084-shares-belong-to-accounts.md), [0140](../decisions/0140-accounts-capability-pair.md)) and memory priorities ([0062](../decisions/0062-memory-priority-capability.md)), and when memory is short it hands out fair shares ([0049](../decisions/0049-memory-fair-share.md)). Programs over their share get notices and deadlines ([0076](../decisions/0076-memory-notice-deadline.md), [0132](../decisions/0132-events-notify-then-call.md)). Supervisors receive reclaim capabilities ([0105](../decisions/0105-supervisor-reclaims.md)), the service reports memory use ([0111](../decisions/0111-resource-accounting.md)), and the first capabilities it issues go to the root task ([0163](../decisions/0163-services-hand-first-capabilities.md)).

- **Implements:** [0012](../decisions/0012-no-memory-cap.md), [0023](../decisions/0023-no-quotas.md), [0025](../decisions/0025-user-mode-memory.md), [0048](../decisions/0048-reserve-on-grant.md), [0049](../decisions/0049-memory-fair-share.md), [0050](../decisions/0050-clients-pay.md), [0062](../decisions/0062-memory-priority-capability.md), [0064](../decisions/0064-memory-belongs-to-payer.md), [0076](../decisions/0076-memory-notice-deadline.md), [0084](../decisions/0084-shares-belong-to-accounts.md), [0105](../decisions/0105-supervisor-reclaims.md), [0111](../decisions/0111-resource-accounting.md), [0132](../decisions/0132-events-notify-then-call.md), [0137](../decisions/0137-memory-capability-per-program.md), [0140](../decisions/0140-accounts-capability-pair.md), [0163](../decisions/0163-services-hand-first-capabilities.md).
- **Interfaces:** the memory service's protocol.
- **Tests:** a request over its share fails while memory is short ([0049](../decisions/0049-memory-fair-share.md)), and a program that ignores a notice is stopped at its deadline ([0076](../decisions/0076-memory-notice-deadline.md)). Nothing caps a program while memory is free ([0012](../decisions/0012-no-memory-cap.md), [0023](../decisions/0023-no-quotas.md)). In the attack test, a client that floods a service with state pays for it ([0050](../decisions/0050-clients-pay.md)).

#### 13. The default pager, the loader and the boot manifest

Needs step 12. The default pager handles reservations and backing ([0138](../decisions/0138-pager-keeps-reservations.md)), low addresses on request ([0078](../decisions/0078-low-addresses-on-request.md)) and stacks that grow ([0048](../decisions/0048-reserve-on-grant.md)). It declines faults to exception handlers ([0106](../decisions/0106-pager-passes-on-faults.md)), never maps the lowest 64 KiB ([0074](../decisions/0074-low-64k-unmapped.md)), and retries after the window's batches ([0217](../decisions/0217-window-flush-batched.md)).

The loader reads ELF as a container ([0038](../decisions/0038-elf-container.md), [0039](../decisions/0039-native-per-arch.md)) and accepts relative relocations only ([0211](../decisions/0211-relative-relocations.md)). It places programs without randomization ([0061](../decisions/0061-no-randomization-yet.md), [0070](../decisions/0070-position-independent-programs.md)) and takes the image's highest address ([0212](../decisions/0212-low-image-declaration.md)), the declarations ([0147](../decisions/0147-elf-declarations-only.md)) and the predictor-flush flag ([0136](../decisions/0136-predictor-flush-flag.md)). It also writes the startup data ([0107](../decisions/0107-loader-writes-startup-data.md), [0108](../decisions/0108-startup-information.md), [0146](../decisions/0146-startup-data-block.md)), assigns process numbers ([0179](../decisions/0179-command-copies-namespace.md)), maps in the system library ([0117](../decisions/0117-system-library.md)), and sets up the namespace ([0028](../decisions/0028-namespace-at-spawn.md), [0131](../decisions/0131-namespace-path-prefixes.md)).

Beyond those come the boot manifest, with the root task starting the ramdisk's other programs through the loader ([0160](../decisions/0160-root-task-starts-four.md)); regions, handover and fill-and-report in the bindings ([0058](../decisions/0058-shared-and-handed-over.md), [0059](../decisions/0059-caller-allocates.md), [0139](../decisions/0139-results-fill-and-report.md)); the accessors ([0065](../decisions/0065-lost-memory-is-an-error.md), [0133](../decisions/0133-recovery-point-in-register.md)); and lending to services ([0050](../decisions/0050-clients-pay.md)).

- **Implements:** [0022](../decisions/0022-explicit-spawn.md), [0027](../decisions/0027-user-mode-pagers.md), [0028](../decisions/0028-namespace-at-spawn.md), [0031](../decisions/0031-user-mode-loader.md), [0038](../decisions/0038-elf-container.md), [0039](../decisions/0039-native-per-arch.md), [0048](../decisions/0048-reserve-on-grant.md), [0050](../decisions/0050-clients-pay.md), [0056](../decisions/0056-pager-chosen-at-spawn.md), [0057](../decisions/0057-pager-fault-chain.md), [0058](../decisions/0058-shared-and-handed-over.md), [0059](../decisions/0059-caller-allocates.md), [0061](../decisions/0061-no-randomization-yet.md), [0065](../decisions/0065-lost-memory-is-an-error.md), [0070](../decisions/0070-position-independent-programs.md), [0074](../decisions/0074-low-64k-unmapped.md), [0078](../decisions/0078-low-addresses-on-request.md), [0106](../decisions/0106-pager-passes-on-faults.md), [0107](../decisions/0107-loader-writes-startup-data.md), [0108](../decisions/0108-startup-information.md), [0117](../decisions/0117-system-library.md), [0131](../decisions/0131-namespace-path-prefixes.md), [0133](../decisions/0133-recovery-point-in-register.md), [0136](../decisions/0136-predictor-flush-flag.md), [0138](../decisions/0138-pager-keeps-reservations.md), [0139](../decisions/0139-results-fill-and-report.md), [0146](../decisions/0146-startup-data-block.md), [0147](../decisions/0147-elf-declarations-only.md), [0155](../decisions/0155-ramdisk-reaches-storage.md), [0160](../decisions/0160-root-task-starts-four.md), [0179](../decisions/0179-command-copies-namespace.md), [0195](../decisions/0195-bindings-safe-output.md), [0211](../decisions/0211-relative-relocations.md), [0212](../decisions/0212-low-image-declaration.md), [0217](../decisions/0217-window-flush-batched.md).
- **Interfaces:** the pager's, the loader's and the boot manifest's.
- **Tests:** a program with other relocations is refused ([0211](../decisions/0211-relative-relocations.md)), and a child holds only what it was given ([0022](../decisions/0022-explicit-spawn.md)). A service that loses a client's lent memory keeps serving the others ([0065](../decisions/0065-lost-memory-is-an-error.md)). Layouts repeat from boot to boot ([0061](../decisions/0061-no-randomization-yet.md)).

#### 14. The log, time and random services, and std's second stage

Needs step 13. The log service lives in the ramdisk ([0253](../decisions/0253-ramdisk-log-and-clocks.md)); it takes over the second serial port and shows logs on the framebuffer ([0236](../decisions/0236-program-output-second-serial.md)). The time service starts right after the loader, with monotonic time from the counter ([0112](../decisions/0112-time-service.md), [0113](../decisions/0113-counter-capability.md), [0173](../decisions/0173-time-service-entry.md), [0252](../decisions/0252-time-service-starts-early.md)). The random service is seeded from the CPU and the firmware ([0174](../decisions/0174-random-service-and-seed.md)), and our fork of the boot loader passes on the firmware's seed.

std's second stage covers threads, parking and locks ([0177](../decisions/0177-locks-on-parking.md)), sleep, thread-locals, time, randomness, standard streams ([0180](../decisions/0180-streams-and-exit-status.md)), and unwinding ([0109](../decisions/0109-panic-strategy.md)).

- **Implements:** [0003](../decisions/0003-rust-std-no-libc.md), [0109](../decisions/0109-panic-strategy.md), [0112](../decisions/0112-time-service.md), [0113](../decisions/0113-counter-capability.md), [0173](../decisions/0173-time-service-entry.md), [0174](../decisions/0174-random-service-and-seed.md), [0177](../decisions/0177-locks-on-parking.md), [0180](../decisions/0180-streams-and-exit-status.md), [0182](../decisions/0182-all-of-std.md), [0229](../decisions/0229-std-stages.md), [0236](../decisions/0236-program-output-second-serial.md), [0252](../decisions/0252-time-service-starts-early.md), [0253](../decisions/0253-ramdisk-log-and-clocks.md).
- **Designs:** the log service ([0226](../decisions/0226-open-topics-at-their-step.md)).
- **Tests:** upstream's std tests for stage 2, and programs without `svc/time` or `svc/random` behaving as [0173](../decisions/0173-time-service-entry.md) and [0174](../decisions/0174-random-service-and-seed.md) say.

Checkpoint 4: the laptop runs the core services, and the log service shows their logs on the framebuffer.

### Phase 5. Multicore and devices

#### 15. Hardware discovery and multicore

Needs step 14. The hardware-discovery service reads ACPI's tables and AML, and the device tree ([0032](../decisions/0032-minimal-hardware-discovery.md)). It produces the list of CPUs ([0097](../decisions/0097-user-mode-starts-cores.md), [0165](../decisions/0165-core-objects.md)) and the values for powering off ([0213](../decisions/0213-power-capability.md)), and it finds out whether there's an IOMMU ([0171](../decisions/0171-no-iommu-trusted-drivers.md)).

Cores start through core objects ([0165](../decisions/0165-core-objects.md)), with startup IPIs through the low page and PSCI's CPU_ON; revoke can take a core offline ([0166](../decisions/0166-core-revoke-offline.md)). The scheduler now runs on every core ([0093](../decisions/0093-scheduler-per-core.md)) and pairs sibling hardware threads by core-sharing group ([0034](../decisions/0034-smt-trusted-pairs.md), [0098](../decisions/0098-core-sharing-capability.md), [0099](../decisions/0099-kernel-beside-sibling.md), [0135](../decisions/0135-core-sharing-from-scheduler.md)). Multicore also means the kernel lock under contention ([0095](../decisions/0095-one-kernel-lock.md), [0218](../decisions/0218-lock-yield.md)) and cross-core flushes ([0053](../decisions/0053-revoke-destroys-derived.md), [0217](../decisions/0217-window-flush-batched.md)).

- **Implements:** [0032](../decisions/0032-minimal-hardware-discovery.md), [0034](../decisions/0034-smt-trusted-pairs.md), [0053](../decisions/0053-revoke-destroys-derived.md), [0093](../decisions/0093-scheduler-per-core.md), [0095](../decisions/0095-one-kernel-lock.md), [0097](../decisions/0097-user-mode-starts-cores.md), [0098](../decisions/0098-core-sharing-capability.md), [0099](../decisions/0099-kernel-beside-sibling.md), [0100](../decisions/0100-cpu-flaw-mitigations.md), [0135](../decisions/0135-core-sharing-from-scheduler.md), [0165](../decisions/0165-core-objects.md), [0166](../decisions/0166-core-revoke-offline.md), [0171](../decisions/0171-no-iommu-trusted-drivers.md), [0213](../decisions/0213-power-capability.md), [0217](../decisions/0217-window-flush-batched.md), [0218](../decisions/0218-lock-yield.md), [0240](../decisions/0240-timing-on-laptop.md), [0246](../decisions/0246-interruption-stretch-limit.md).
- **Designs:** the hardware-discovery service's interface ([0226](../decisions/0226-open-topics-at-their-step.md)).
- **Tests:** QEMU runs with several cores and SMT topologies, and we revoke a running core's memory ([0166](../decisions/0166-core-revoke-offline.md)). Attacks work across cores: revokes that would hold the lock, and flush loops. The 20 µs budget is checked on the laptop ([0240](../decisions/0240-timing-on-laptop.md), [0246](../decisions/0246-interruption-stretch-limit.md)).

#### 16. The device manager, the PCI bus driver and the IOMMU service

Needs step 15. First the device manager ([0172](../decisions/0172-device-manager.md)) and the PCI bus driver, which keeps configuration space and turns on switches' access controls ([0215](../decisions/0215-pci-bus-driver-config.md)). The IOMMU service covers VT-d and SMMUv3, with interrupt remapping ([0169](../decisions/0169-iommu-service.md), [0214](../decisions/0214-interrupt-remapping.md)), and the GIC's ITS lives in the kernel ([0214](../decisions/0214-interrupt-remapping.md)). Memory is marked for devices and held back after a revoke ([0170](../decisions/0170-dma-quarantine.md)). Images can also run without an IOMMU, and a boot setting can require one ([0171](../decisions/0171-no-iommu-trusted-drivers.md)). Drivers get interrupt capabilities ([0167](../decisions/0167-interrupts-as-notifications.md)).

The clock drivers sit in the ramdisk ([0253](../decisions/0253-ramdisk-log-and-clocks.md)). They cover the CMOS clock and the PL031, and they give the time service its wall-clock time ([0173](../decisions/0173-time-service-entry.md), [0252](../decisions/0252-time-service-starts-early.md)).

- **Implements:** [0167](../decisions/0167-interrupts-as-notifications.md), [0168](../decisions/0168-device-memory-and-ports.md), [0169](../decisions/0169-iommu-service.md), [0170](../decisions/0170-dma-quarantine.md), [0171](../decisions/0171-no-iommu-trusted-drivers.md), [0172](../decisions/0172-device-manager.md), [0173](../decisions/0173-time-service-entry.md), [0214](../decisions/0214-interrupt-remapping.md), [0215](../decisions/0215-pci-bus-driver-config.md), [0252](../decisions/0252-time-service-starts-early.md), [0253](../decisions/0253-ramdisk-log-and-clocks.md).
- **Designs:** the device manager's matching, and the drivers' protocols; the regions that the firmware reserves for devices, such as VT-d's RMRR ([0223](../decisions/0223-qemu-and-laptop-checkpoints.md), [0226](../decisions/0226-open-topics-at-their-step.md)).
- **Tests:** the attacks are edu's DMA outside its mapping, which is refused, and its MSI, which reaches only its own interrupt ([0169](../decisions/0169-iommu-service.md), [0214](../decisions/0214-interrupt-remapping.md)). Configuration writes that move a device's ranges or turn on ATS are refused ([0215](../decisions/0215-pci-bus-driver-config.md)). Memory mapped for a device is held back after a revoke until the IOMMU service releases it ([0170](../decisions/0170-dma-quarantine.md)), and images run without an IOMMU ([0171](../decisions/0171-no-iommu-trusted-drivers.md)).

Checkpoint 5: the laptop runs on all its cores, with VT-d confining its devices, and lists its PCI devices.

### Phase 6. Storage, packages and verified boot

#### 17. NVMe, block storage and the filesystem

Needs step 16. The step builds the NVMe driver ([0224](../decisions/0224-laptop-usb-storage.md)) and the block-storage service, which handles GPT partitions, among them a swap partition's type ([0185](../decisions/0185-swap-partition.md)). On top sits the filesystem service, with its on-disk format, directories that have owners and grants ([0183](../decisions/0183-access-on-directories.md)), files that record their creators, paths resolved from capabilities ([0030](../decisions/0030-filesystem-decides-access.md), [0176](../decisions/0176-paths-and-current-directory.md)), and read-only pages shared from its cache ([0184](../decisions/0184-shared-file-pages.md)).

- **Implements:** [0030](../decisions/0030-filesystem-decides-access.md), [0050](../decisions/0050-clients-pay.md), [0155](../decisions/0155-ramdisk-reaches-storage.md), [0183](../decisions/0183-access-on-directories.md), [0184](../decisions/0184-shared-file-pages.md), [0185](../decisions/0185-swap-partition.md), [0224](../decisions/0224-laptop-usb-storage.md).
- **Designs:** the filesystem's on-disk format and protocol; the block protocol, the partition types and the disk layout ([0226](../decisions/0226-open-topics-at-their-step.md)).
- **Tests:** crash consistency, with the power cut in QEMU in the middle of writes, and grants on directories ([0183](../decisions/0183-access-on-directories.md)). In the attack test, a client that floods the filesystem pays for it ([0050](../decisions/0050-clients-pay.md)).

#### 18. USB and USB storage

This step also needs step 16. It builds the xHCI driver, the USB core, USB mass storage and USB keyboards ([0224](../decisions/0224-laptop-usb-storage.md)), plus laptop images that start no driver for the internal disk ([0224](../decisions/0224-laptop-usb-storage.md)).

- **Implements:** [0224](../decisions/0224-laptop-usb-storage.md).
- **Designs:** the USB drivers' protocols ([0226](../decisions/0226-open-topics-at-their-step.md)).
- **Tests:** QEMU's xHCI controller, with USB storage and a USB keyboard, on both architectures.

#### 19. Packages, and std's third stage

Needs step 17. We start with the package format and its tools ([0203](../decisions/0203-package-format.md)) and the signatures ([0204](../decisions/0204-hybrid-signatures.md)). The package store handles installs, checks after each boot, checks of every block, and the owner's approvals ([0041](../decisions/0041-immutable-packages.md), [0206](../decisions/0206-owner-approves-signers.md), [0207](../decisions/0207-install-and-start-checks.md)). It also deals with declared std versions ([0210](../decisions/0210-std-version-declared.md)), loads the rest of the system image from storage ([0155](../decisions/0155-ramdisk-reaches-storage.md)), and serves code pages with the execute right ([0054](../decisions/0054-kernel-enforces-execute.md), [0184](../decisions/0184-shared-file-pages.md)). At each start, the loader checks the owner's policy ([0207](../decisions/0207-install-and-start-checks.md)).

std's third stage brings files, paths and the current directory ([0176](../decisions/0176-paths-and-current-directory.md)), and processes, exit statuses and pipes ([0179](../decisions/0179-command-copies-namespace.md), [0180](../decisions/0180-streams-and-exit-status.md)). The `thisos` crate and `std::os::thisos` arrive with it ([0178](../decisions/0178-native-api-crate.md)).

- **Implements:** [0003](../decisions/0003-rust-std-no-libc.md), [0040](../decisions/0040-signed-programs-owner-policy.md), [0041](../decisions/0041-immutable-packages.md), [0042](../decisions/0042-updates.md), [0054](../decisions/0054-kernel-enforces-execute.md), [0104](../decisions/0104-supervisor.md), [0155](../decisions/0155-ramdisk-reaches-storage.md), [0176](../decisions/0176-paths-and-current-directory.md), [0178](../decisions/0178-native-api-crate.md), [0179](../decisions/0179-command-copies-namespace.md), [0180](../decisions/0180-streams-and-exit-status.md), [0182](../decisions/0182-all-of-std.md), [0184](../decisions/0184-shared-file-pages.md), [0203](../decisions/0203-package-format.md), [0204](../decisions/0204-hybrid-signatures.md), [0206](../decisions/0206-owner-approves-signers.md), [0207](../decisions/0207-install-and-start-checks.md), [0210](../decisions/0210-std-version-declared.md), [0229](../decisions/0229-std-stages.md).
- **Designs:** where the package store keeps packages ([0226](../decisions/0226-open-topics-at-their-step.md)).
- **Tests:** upstream's std tests for stage 3, and attacks, in which a tampered block or manifest is caught ([0203](../decisions/0203-package-format.md), [0207](../decisions/0207-install-and-start-checks.md)).

#### 20. Verified boot and updates

Needs step 19. The work starts in our fork of the boot loader, which checks the kernel and the ramdisk against the image's signature ([0156](../decisions/0156-boot-loader-checks-image.md)), reads the root key's statements ([0205](../decisions/0205-root-and-working-keys.md)), and selects an image with tries and fallback ([0158](../decisions/0158-image-try-count.md)). On removable media it leaves the firmware's boot entries alone ([0208](../decisions/0208-boot-loader-beside-old.md), [0223](../decisions/0223-qemu-and-laptop-checkpoints.md)).

The update service installs images and marks them good, and installs boot loaders beside the old one ([0042](../decisions/0042-updates.md), [0158](../decisions/0158-image-try-count.md), [0208](../decisions/0208-boot-loader-beside-old.md)). The step also brings the signing tools and test keys, and Secure Boot in QEMU with an enrolled key ([0205](../decisions/0205-root-and-working-keys.md)).

- **Implements:** [0042](../decisions/0042-updates.md), [0156](../decisions/0156-boot-loader-checks-image.md), [0158](../decisions/0158-image-try-count.md), [0202](../decisions/0202-offline-attackers.md), [0204](../decisions/0204-hybrid-signatures.md), [0205](../decisions/0205-root-and-working-keys.md), [0208](../decisions/0208-boot-loader-beside-old.md), [0209](../decisions/0209-no-rollback-protection.md).
- **Designs:** where updates come from, and how the boot loader tells removable media apart ([0226](../decisions/0226-open-topics-at-their-step.md)).
- **Tests:** in the attacks, a tampered kernel, ramdisk or statement is refused ([0156](../decisions/0156-boot-loader-checks-image.md), [0205](../decisions/0205-root-and-working-keys.md)), and so is a revoked working key, by a newer boot loader ([0205](../decisions/0205-root-and-working-keys.md)). A bad image falls back after its tries ([0158](../decisions/0158-image-try-count.md)), and an older signed image still boots ([0209](../decisions/0209-no-rollback-protection.md)).

Checkpoint 6: the laptop boots from a USB stick, with its storage on a USB disk ([0224](../decisions/0224-laptop-usb-storage.md)).

### Phase 7. Users, the network and the first release

#### 21. The console, login and the shell

Needs steps 18 and 19. The console works with the framebuffer, a PS/2 keyboard and USB keyboards ([0223](../decisions/0223-qemu-and-laptop-checkpoints.md)). The login service covers authentication, sessions with their accounts ([0140](../decisions/0140-accounts-capability-pair.md)) and identity capabilities ([0141](../decisions/0141-identity-capability.md)), stable user IDs ([0183](../decisions/0183-access-on-directories.md)), and the owner, who approves signers ([0206](../decisions/0206-owner-approves-signers.md)). The step ends with a shell and a system monitor that reads the services' reports ([0111](../decisions/0111-resource-accounting.md)).

- **Implements:** [0029](../decisions/0029-users-as-capabilities.md), [0030](../decisions/0030-filesystem-decides-access.md), [0111](../decisions/0111-resource-accounting.md), [0140](../decisions/0140-accounts-capability-pair.md), [0141](../decisions/0141-identity-capability.md), [0183](../decisions/0183-access-on-directories.md), [0206](../decisions/0206-owner-approves-signers.md).
- **Designs:** the console and terminal, how users authenticate, how the owner is set up, and the shell ([0226](../decisions/0226-open-topics-at-their-step.md)).
- **Tests:** a session reaches only what its grants give ([0183](../decisions/0183-access-on-directories.md)), and an identity capability grants nothing by itself ([0141](../decisions/0141-identity-capability.md)).

#### 22. The network, and std's fourth stage

Needs step 19. Here we build the virtio-net driver and the network service, with a TCP/IP stack, name resolution and narrower entries ([0181](../decisions/0181-network-service.md)), followed by std's fourth stage ([0229](../decisions/0229-std-stages.md)).

- **Implements:** [0003](../decisions/0003-rust-std-no-libc.md), [0181](../decisions/0181-network-service.md), [0182](../decisions/0182-all-of-std.md), [0229](../decisions/0229-std-stages.md).
- **Designs:** the network stack, and the network service's protocol ([0226](../decisions/0226-open-topics-at-their-step.md)).
- **Tests:** upstream's std tests for networking. A program without `svc/net` has no network ([0181](../decisions/0181-network-service.md)).

#### 23. Third-party programs

Needs steps 20, 21 and 22. The step publishes the target specs, std's patch and the `thisos` crate ([0178](../decisions/0178-native-api-crate.md), [0187](../decisions/0187-pinned-stable-toolchain.md), [0195](../decisions/0195-bindings-safe-output.md)). Developers get packaging and signing, and the owner approves their keys ([0206](../decisions/0206-owner-approves-signers.md)); packages can be installed ([0207](../decisions/0207-install-and-start-checks.md)). The list of std versions with known flaws is kept here too ([0210](../decisions/0210-std-version-declared.md)). Last, version 1 of the interfaces that third-party programs reach is released ([0148](../decisions/0148-freeze-covers-rules.md), [0196](../decisions/0196-released-interface-copies.md), [0230](../decisions/0230-interfaces-released-with-sdk.md)).

- **Implements:** [0143](../decisions/0143-add-only-versioning.md), [0148](../decisions/0148-freeze-covers-rules.md), [0178](../decisions/0178-native-api-crate.md), [0187](../decisions/0187-pinned-stable-toolchain.md), [0195](../decisions/0195-bindings-safe-output.md), [0196](../decisions/0196-released-interface-copies.md), [0206](../decisions/0206-owner-approves-signers.md), [0207](../decisions/0207-install-and-start-checks.md), [0210](../decisions/0210-std-version-declared.md), [0230](../decisions/0230-interfaces-released-with-sdk.md).
- **Designs:** the SDK's form, and how it's published ([0226](../decisions/0226-open-topics-at-their-step.md)).
- **Tests:** a program built outside the repository, with only the published pieces, installs, runs and unwinds, and the compatibility checker guards every released copy ([0196](../decisions/0196-released-interface-copies.md)).

#### 24. The first release

Needs step 23. The project's root key and working keys are made offline ([0205](../decisions/0205-root-and-working-keys.md)). A second machine rebuilds the first system image and boot loader before they're signed ([0241](../decisions/0241-reproducibility-checks.md)), and the release notes state the gaps that later work closes ([0251](../decisions/0251-later-work.md)).

- **Implements:** [0194](../decisions/0194-reproducible-builds.md), [0205](../decisions/0205-root-and-working-keys.md), [0241](../decisions/0241-reproducibility-checks.md), [0251](../decisions/0251-later-work.md).
- **Designs:** how the root key is made and kept ([0226](../decisions/0226-open-topics-at-their-step.md)).
- **Tests:** the full suite on both architectures, with and without an IOMMU.
Checkpoint 7: the laptop runs the whole system, apart from its network.

## Open topics

We design each open topic at the step that needs it, and settle it with an ordinary record ([0226](../decisions/0226-open-topics-at-their-step.md)). Interfaces are different: they're designed in the IDL as their steps build them, and they need no record until they're released ([0148](../decisions/0148-freeze-covers-rules.md), [0230](../decisions/0230-interfaces-released-with-sdk.md)). A choice that nothing else depends on needs no record either ([0073](../decisions/0073-kernel-layout-in-implementation.md)).

| Open topic | Step | Records |
|---|---|---|
| How tests name the records they check | 1 | [0234](../decisions/0234-tests-cite-records.md) |
| The project's set of lints | 1 | [0248](../decisions/0248-coding-and-doc-standards.md) |
| The CI workflows, and the nightly runs | 1 | [0237](../decisions/0237-public-repository-and-ci.md) |
| The image's version numbers | 2 | [0209](../decisions/0209-no-rollback-protection.md) |
| How the root task finds the second serial port before hardware discovery runs | 8 | [0236](../decisions/0236-program-output-second-serial.md) |
| The log service | 14 | [0152](../decisions/0152-kernel-log-and-reset.md), [0180](../decisions/0180-streams-and-exit-status.md), [0236](../decisions/0236-program-output-second-serial.md) |
| The hardware-discovery service's interface | 15 | [0032](../decisions/0032-minimal-hardware-discovery.md), [0097](../decisions/0097-user-mode-starts-cores.md), [0165](../decisions/0165-core-objects.md) |
| The device manager's matching, and the drivers' protocols | 16 | [0172](../decisions/0172-device-manager.md) |
| The regions that the firmware reserves for devices, such as VT-d's RMRR | 16 | [0169](../decisions/0169-iommu-service.md), [0223](../decisions/0223-qemu-and-laptop-checkpoints.md) |
| The filesystem's on-disk format and protocol | 17 | [0030](../decisions/0030-filesystem-decides-access.md), [0183](../decisions/0183-access-on-directories.md), [0184](../decisions/0184-shared-file-pages.md) |
| The block protocol, the partition types and the disk layout | 17 | [0185](../decisions/0185-swap-partition.md) |
| The USB drivers' protocols | 18 | [0224](../decisions/0224-laptop-usb-storage.md) |
| Where the package store keeps packages | 19 | [0041](../decisions/0041-immutable-packages.md), [0155](../decisions/0155-ramdisk-reaches-storage.md) |
| Where updates come from | 20 | [0042](../decisions/0042-updates.md), [0158](../decisions/0158-image-try-count.md) |
| How the boot loader tells removable media apart | 20 | [0208](../decisions/0208-boot-loader-beside-old.md), [0223](../decisions/0223-qemu-and-laptop-checkpoints.md) |
| The console and terminal | 21 | [0180](../decisions/0180-streams-and-exit-status.md), [0223](../decisions/0223-qemu-and-laptop-checkpoints.md) |
| How users authenticate, and how the owner is set up | 21 | [0029](../decisions/0029-users-as-capabilities.md), [0141](../decisions/0141-identity-capability.md), [0206](../decisions/0206-owner-approves-signers.md) |
| The shell | 21 | [0022](../decisions/0022-explicit-spawn.md), [0179](../decisions/0179-command-copies-namespace.md) |
| The network stack, and the network service's protocol | 22 | [0181](../decisions/0181-network-service.md) |
| The SDK's form, and how it's published | 23 | [0187](../decisions/0187-pinned-stable-toolchain.md), [0195](../decisions/0195-bindings-safe-output.md) |
| How the root key is made and kept | 24 | [0205](../decisions/0205-root-and-working-keys.md) |
| How two maintainers both approve a tier 1 change | When a second maintainer joins | [0238](../decisions/0238-pushes-and-pull-requests.md), [0242](../decisions/0242-review-tiers.md) |

## Modularity

Everything lives in one repository, with a Cargo workspace for each kind of target: host tools, the kernel, the boot loader, and programs for the thisOS targets ([0231](../decisions/0231-repository-layout.md)). Beside the workspaces sit std's patch and the target specs, the interface definitions and their released copies, the vendored crates, the crate reviews and the audit records. [0231](../decisions/0231-repository-layout.md) gives the layout.

The kernel is a generic library crate, plus a crate for each architecture that implements its traits and builds the binary ([0232](../decisions/0232-traits-and-specific-crates.md)). Platform code (interrupt controllers, timers, UARTs and firmware tables) sits in platform crates, and the kernel chooses it at boot from the firmware tables. The simulated machine is a generic crate that implements the same traits on the host ([0233](../decisions/0233-simulated-machine.md)).

User-mode programs are crates in the programs workspace: the system library, the services and the drivers. The libraries they share, such as the page-table library ([0052](../decisions/0052-user-mode-page-tables.md)), the IDL runtime ([0195](../decisions/0195-bindings-safe-output.md)) and the `thisos` crate ([0178](../decisions/0178-native-api-crate.md)), sit together in one directory.

Architecture- and platform-specific code lives only in crates marked as such, and each is named after the generic crate it serves, with a suffix ([0197](../decisions/0197-architecture-check.md), [0231](../decisions/0231-repository-layout.md)). Generic crates state their hardware needs as traits, and never depend on specific crates ([0232](../decisions/0232-traits-and-specific-crates.md)). The build tool knows every workspace and runs every check over all of them ([0190](../decisions/0190-cargo-and-build-tool.md)).

## Testing

The kinds of tests come from [0235](../decisions/0235-kinds-of-tests.md). Every generic crate gets host tests, and that includes the generic kernel on the simulated machine ([0233](../decisions/0233-simulated-machine.md)). Miri and AddressSanitizer run over the audited set's host code. QEMU tests cover the kernel, user-mode programs, std by stage ([0229](../decisions/0229-std-stages.md)) and whole images, with and without an IOMMU. On top of those there are planted violations for every build check, an attack test for each recorded defence, a regression test for every bug fix, and benchmarks.

Output and results follow [0236](../decisions/0236-program-output-second-serial.md). The kernel writes to its own serial port, and programs write to a second one, which the root task drives until the log service takes it over. A test run ends through the power capability.

CI ([0237](../decisions/0237-public-repository-and-ci.md)) means GitHub Actions running the build tool's `ci` command for every change, on both architectures, with x86_64 under KVM and AArch64 emulated. Long fuzzing and stress runs are nightly. Maintainers run the `ci` command locally before they push to main, and contributors send pull requests ([0238](../decisions/0238-pushes-and-pull-requests.md)).

Benchmarks and time budgets run on the development laptop under KVM, at checkpoints and for kernel changes that may affect them ([0240](../decisions/0240-timing-on-laptop.md)).

For reproducibility ([0241](../decisions/0241-reproducibility-checks.md)), every CI run builds the image and the boot loader twice, under varied conditions, and compares the bytes. A second machine rebuilds a release before it's signed.

To cover the design ([0234](../decisions/0234-tests-cite-records.md)), tests name the records they check, and the build tool maps every Accepted record to its step and its tests. The [coverage map](#coverage-map) below gives each record's steps. Regressions are kept out by a test for every fixed bug, and by CI on every change ([0235](../decisions/0235-kinds-of-tests.md), [0238](../decisions/0238-pushes-and-pull-requests.md)).

## Review and kernel assurance

Review has three tiers, by role ([0242](../decisions/0242-review-tiers.md)). Tier 1 is reviewed line by line against a checklist, and covers the kernel, our fork of the boot loader, the audited set, and the build checks that enforce safety. Tier 2 gets a full review against the threat model: the trusted-base services, the random service, the image builder and the signing tools. Tier 3, which is everything else, gets a normal review. No change is approved by its own author.

The audit records ([0243](../decisions/0243-audit-records-and-crate-reviews.md)) start from the list of audited crates, in TOML, with one Markdown record for each crate: its safety arguments, the tests and fuzzing that cover it, and a log of its audits. cargo-vet records the review of every vendored crate, and cargo-deny reads a snapshot of the advisory database. Reviews may be imported from named organizations, except for the audited set ([0244](../decisions/0244-imported-crate-reviews.md)).

On coverage and fuzzing ([0245](../decisions/0245-coverage-and-fuzzing.md)), every line of the generic kernel is measured on the simulated machine, and every function of the architecture and platform code is reached in QEMU. System calls get coverage-guided fuzzing from step 4, and in QEMU from step 9. Each trusted-base service's protocol is fuzzed through its bindings.

The longest stretch ([0246](../decisions/0246-interruption-stretch-limit.md)) is limited by work limits between interruption points, which are constants that the simulator checks, and by a budget of 20 µs on the laptop. That bounds the kernel's interrupt latency ([0089](../decisions/0089-interrupts-off-in-kernel.md)) and the wait for its lock ([0218](../decisions/0218-lock-yield.md)).

Two things keep the kernel small ([0247](../decisions/0247-kernel-size-ratchet.md)): a record for any new kernel mechanism, and a size ratchet. The build tool counts the kernel's code from step 1, and the counts become its budget at the end of step 9.

## Coding and documentation standards

[0248](../decisions/0248-coding-and-doc-standards.md) sets these standards.

The code uses edition 2024 and rustfmt's defaults, with a project set of clippy lints, and warnings fail CI. Overflow checks are on in every build of the kernel and the OS's own programs. No panic in the kernel can be reached from a system call, and lints deny `unwrap`, `expect`, `panic!` and unchecked indexing outside boot code. The kernel also masks indexes and sizes from programs against speculation.

Every unsafe block has a `SAFETY` comment, and every unsafe function a `# Safety` section. Files carry no license header; each manifest gives the license as MIT OR Apache-2.0 ([0199](../decisions/0199-license.md)).

For documentation, each crate opens with what it is, the records it implements and its invariants. Public items are documented, and code that follows a decision cites its record. Documents and comments use plain English and short sentences. A commit message starts with a summary sentence in the past tense, then a paragraph.

## Estimates

A step's estimate is the code it adds, split by review tier, and the review time that code needs ([0249](../decisions/0249-estimates-and-checkpoints.md)). The rates are a first guess: 150 lines an hour in tier 1, 300 in tier 2, and 600 in tier 3. Vendored code that needs our own audit or review is estimated separately. From checkpoint 1 on, the pace we measure on earlier steps replaces these rates, and calendar dates follow from it.

| Step | Tier 1 | Tier 2 | Tier 3 | Vendored code we audit or review | Review hours |
|---|---|---|---|---|---|
| 1 | 2.3k | — | 6.3k | The tools' crates, mostly through imported reviews | 35 |
| 2 | 4.5k | 3.0k | 4.5k | The `bootloader` crate and its dependencies, about 40,000 lines, audited | 200 |
| 3 | 5.0k | — | 2.0k | — | 35 |
| 4 | 4.0k | — | 7.3k | The fuzzing tools' crates | 50 |
| 5 | 4.5k | — | 3.0k | — | 35 |
| 6 | 6.0k | — | 9.0k | A parser for the IDL compiler | 65 |
| 7 | 3.0k | — | 0.5k | — | 20 |
| 8 | 7.5k | 0.5k | 2.0k | — | 55 |
| 9 | 7.0k | — | 6.5k | — | 60 |
| 10 | 4.0k | 6.5k | 3.5k | — | 55 |
| 11 | — | 4.0k | 2.5k | — | 20 |
| 12 | — | 5.0k | 3.0k | — | 20 |
| 13 | 0.6k | 9.0k | 7.0k | — | 45 |
| 14 | 4.3k | 1.0k | 6.5k | An unwinder and a random-number generator, about 7,000 lines | 60 |
| 15 | 2.5k | 2.0k | 7.0k | ACPI and device-tree parsers, about 17,000 lines | 60 |
| 16 | 2.0k | 13.0k | 5.6k | — | 65 |
| 17 | — | 10.0k | 10.5k | — | 50 |
| 18 | — | — | 12.0k | — | 20 |
| 19 | 6.0k | 7.3k | 6.0k | Signature and hash crates, about 25,000 lines, audited | 135 |
| 20 | 2.0k | 5.5k | 4.0k | — | 40 |
| 21 | — | 4.0k | 10.6k | A password-hashing crate, if login uses passwords | 35 |
| 22 | 2.0k | — | 8.5k | A TCP/IP stack, about 30,000 lines | 60 |
| 23 | 0.5k | 1.0k | 4.5k | — | 15 |
| 24 | — | 0.5k | 1.0k | — | 5 |
| **All** | **68k** | **72k** | **133k** | | **1,230** |

Toolchain upgrades are recurring work that runs beside the steps ([0188](../decisions/0188-upgrade-every-second-release.md)). Each upgrade audits the changes to unsafe code in `core`, `alloc` and std, moves std's patch to the new release, rebuilds our fork of the boot loader, and passes the full tests. The first one falls due with Rust 1.101.

## Checkpoints

A checkpoint closes each phase ([0249](../decisions/0249-estimates-and-checkpoints.md)). At one, we compare the actual sizes and times with the estimates and re-estimate the remaining steps, the laptop runs that phase's image ([0223](../decisions/0223-qemu-and-laptop-checkpoints.md)), and the kernel's size counts get reviewed ([0247](../decisions/0247-kernel-size-ratchet.md)). The results are recorded here.

| Checkpoint | After step | On the laptop | Result |
|---|---|---|---|
| 1 | 3 | The kernel boots from a USB stick and shows its log | Not yet |
| 2 | 6 | The kernel's tests run | Not yet |
| 3 | 9 | The first user-mode program runs its tests | Not yet |
| 4 | 14 | The core services run, with their logs on the framebuffer | Not yet |
| 5 | 16 | Every core runs, with VT-d confining the devices | Not yet |
| 6 | 20 | The system boots with its storage on a USB disk | Not yet |
| 7 | 24 | The whole system runs, apart from its network | Not yet |

## Changing the plan

Once the design is frozen, a frozen decision changes only through a record that cites what building found, and names the step it affects ([0201](../decisions/0201-changes-after-freeze.md)). The same holds for adding, removing, splitting, merging or reordering steps, and for moving a decision to another step ([0250](../decisions/0250-plan-changes-after-freeze.md)). Estimates, progress and the dates of checkpoints are the exception: they change at checkpoints, without records ([0249](../decisions/0249-estimates-and-checkpoints.md)). Open topics get ordinary records at their steps ([0226](../decisions/0226-open-topics-at-their-step.md)), and later work gets one when a checkpoint schedules it ([0251](../decisions/0251-later-work.md)).

## Later work

Nothing here is scheduled ([0251](../decisions/0251-later-work.md)), though a checkpoint may schedule an item through an ordinary record. The first release states the gaps that later work closes. Two examples are stored data kept in the clear ([0202](../decisions/0202-offline-attackers.md)) and older signed images that still boot ([0209](../decisions/0209-no-rollback-protection.md)).

| Item | Records | Needs first | Earliest after step |
|---|---|---|---|
| A WebAssembly runtime, with WASI on the native API | [0006](../decisions/0006-non-rust-software.md), [0186](../decisions/0186-non-rust-route.md) | All of std, and the capability to make memory executable ([0043](../decisions/0043-executable-memory-capability.md)) | 22 |
| Native programs in other languages, with the IDL's back ends for them | [0186](../decisions/0186-non-rust-route.md) | The SDK | 23 |
| Randomizing programs, and the kernel | [0061](../decisions/0061-no-randomization-yet.md) | The loader, and a relocatable kernel ([0070](../decisions/0070-position-independent-programs.md), [0189](../decisions/0189-kernel-none-targets.md)) | 13 |
| Swap, encrypted, to a swap partition | [0077](../decisions/0077-swap-in-default-pager.md), [0185](../decisions/0185-swap-partition.md) | Block storage and the default pager | 17 |
| Hardware pointer tagging | [0079](../decisions/0079-pointer-tagging-off.md) | A new record | 8 |
| Formal proofs of parts of the kernel | [0018](../decisions/0018-kernel-assurance.md) | A kernel whose mechanisms are all in | 9 |
| The Rust targets offered to rust-lang/rust | [0187](../decisions/0187-pinned-stable-toolchain.md) | std's patch, complete | 22 |
| Our changes to the `bootloader` crate offered upstream | [0150](../decisions/0150-bootloader-aarch64.md) | The fork's changes | 7, and 20 for the rest |
| Encrypted stored data, with keys from the login service and a key bound to the machine | [0202](../decisions/0202-offline-attackers.md) | The filesystem and the login service | 21 |
| A signature from Microsoft's UEFI certificate authority for the boot loader | [0205](../decisions/0205-root-and-working-keys.md) | A released boot loader | 24 |
| Rollback protection | [0209](../decisions/0209-no-rollback-protection.md) | Verified boot | 20 |
| The 16 and 64 KiB granules on AArch64 | [0069](../decisions/0069-all-page-sizes.md) | AArch64's paging | 8 |
| Discardable memory | [0076](../decisions/0076-memory-notice-deadline.md) | The memory service | 12 |
| Finer-grained locking in the kernel | [0095](../decisions/0095-one-kernel-lock.md) | Multicore | 15 |
| IPC channels, and larger messages | [0118](../decisions/0118-calls-and-notifications.md), [0119](../decisions/0119-message-eight-words.md) | IPC | 9 |
| Correcting the wall clock from network time | [0173](../decisions/0173-time-service-entry.md) | The network | 22 |
| File mappings that write back | [0184](../decisions/0184-shared-file-pages.md) | The filesystem | 17 |
| Self-hosting: building thisOS on thisOS | [0003](../decisions/0003-rust-std-no-libc.md) | A route for C++ programs ([0186](../decisions/0186-non-rust-route.md)) | 23 |
| Stronger checks of the toolchain's output, such as reviewing the kernel's machine code | [0037](../decisions/0037-kernel-toolchain.md) | A kernel whose mechanisms are all in | 9 |
| Real hardware beyond the laptop, and the laptop's Wi-Fi | [0223](../decisions/0223-qemu-and-laptop-checkpoints.md) | USB, and the network | 22 |

## Coverage map

This table lists every Accepted record, the steps that build it and what checks it ([0234](../decisions/0234-tests-cite-records.md)). Superseded records are left out, and "All" marks a record that every step follows.

| Record | Steps | Checked by |
|---|---|---|
| [0001](../decisions/0001-general-purpose-os.md) General-purpose production OS | All | Not testable: it sets the project's purpose |
| [0003](../decisions/0003-rust-std-no-libc.md) Rust is the system language; the Rust standard library replaces libc | 10, 14, 19, 22 | Std tests |
| [0004](../decisions/0004-not-unix.md) Not Unix-like: `target_family` is not `unix` | 10 | Build check: the target specs |
| [0006](../decisions/0006-non-rust-software.md) Non-Rust software is supported later; route still open | Later | Not testable: later work |
| [0008](../decisions/0008-architecture-neutral-design.md) Architecture-neutral design | 1 | Build check: the architecture check |
| [0009](../decisions/0009-x86-64-first.md) x86_64 is the first architecture | 2, 7 | Not testable: an order of work |
| [0010](../decisions/0010-64-bit-only.md) 64-bit architectures only | 1 | Not testable: only 64-bit targets exist |
| [0011](../decisions/0011-aarch64-second.md) AArch64 is the second architecture | 7, 8 | QEMU tests on AArch64 |
| [0012](../decisions/0012-no-memory-cap.md) No fixed limit on program memory | 12 | QEMU tests: a program can use all free memory |
| [0014](../decisions/0014-microkernel.md) Microkernel: a minimal, audited kernel in kernel mode; everything else in user mode | All | Review, and the size count ([0247](../decisions/0247-kernel-size-ratchet.md)) |
| [0015](../decisions/0015-clean-apis.md) APIs and ABIs are designed for this system: inspired by others, never bound to them | All | Review |
| [0016](../decisions/0016-threat-model-scope.md) Threat model: speculative leaks between programs and denial of service are in scope | All | Attack tests |
| [0017](../decisions/0017-unsafe-code-kernel-and-services.md) Unsafe code: one closed, audited set for the kernel and the OS's own services | 1 | Build check: the compiler wrapper |
| [0018](../decisions/0018-kernel-assurance.md) Kernel assurance: review, tests and fuzzing from the start; formal proofs later | 1, 4, 9 | Review ([0242](../decisions/0242-review-tiers.md)), coverage and fuzzing ([0245](../decisions/0245-coverage-and-fuzzing.md)) |
| [0019](../decisions/0019-kernel-minimality.md) Kernel minimality: anything that can reasonably run in user mode does | 1, 9 | The record rule and the size count ([0247](../decisions/0247-kernel-size-ratchet.md)) |
| [0020](../decisions/0020-capabilities.md) Authority comes from capabilities | 4 | Host tests |
| [0021](../decisions/0021-ipc-simple-upgradable.md) IPC starts simple and modular, and is designed to be upgraded | 6, 9 | QEMU tests: programs reach IPC only through the system library |
| [0022](../decisions/0022-explicit-spawn.md) Programs start other programs by explicit spawn | 13 | QEMU tests: a child holds only what it was given |
| [0023](../decisions/0023-no-quotas.md) No quotas: a program's resources are limited only by the hardware and what's available | 11, 12 | QEMU tests: nothing caps a program while resources are free |
| [0024](../decisions/0024-root-task.md) At boot, one root task receives every initial capability | 8 | QEMU tests: the initial capabilities |
| [0025](../decisions/0025-user-mode-memory.md) Physical memory is managed from user mode | 4, 12 | Host and QEMU tests |
| [0026](../decisions/0026-programs-pay-kernel-objects.md) Programs pay for their kernel objects with memory they hold | 4 | Host tests |
| [0027](../decisions/0027-user-mode-pagers.md) Page faults are handled by user-mode pagers | 9, 13 | QEMU tests |
| [0028](../decisions/0028-namespace-at-spawn.md) Programs find services through a namespace handed over at spawn | 13 | QEMU tests |
| [0029](../decisions/0029-users-as-capabilities.md) A user is the set of capabilities a login service grants | 21 | QEMU tests |
| [0030](../decisions/0030-filesystem-decides-access.md) The filesystem service decides who may open stored data | 17, 21 | QEMU tests |
| [0031](../decisions/0031-user-mode-loader.md) The program loader runs in user mode, and starting programs needs a capability | 5, 13 | Host and QEMU tests |
| [0032](../decisions/0032-minimal-hardware-discovery.md) The kernel reads only the hardware information it needs to start | 3, 8, 15 | Kernel tests, and review of platform code |
| [0033](../decisions/0033-cpu-fairness.md) CPU fairness: fair sharing within each priority, and priorities as capabilities | 5, 11 | QEMU tests: shares and priorities |
| [0034](../decisions/0034-smt-trusted-pairs.md) Only programs that trust each other share a CPU core | 15 | QEMU tests with SMT topologies |
| [0036](../decisions/0036-ipc-message-max.md) One IPC message has a small fixed maximum size; larger data goes through shared memory | 9 | Host tests |
| [0037](../decisions/0037-kernel-toolchain.md) The kernel is built with one pinned, audited toolchain | 1 | Build check: the pinned toolchain |
| [0038](../decisions/0038-elf-container.md) Programs ship as ELF, used only as a file container | 13 | QEMU tests |
| [0039](../decisions/0039-native-per-arch.md) Programs ship as native code, built separately for each architecture | 13 | QEMU tests on both architectures |
| [0040](../decisions/0040-signed-programs-owner-policy.md) Programs are signed, and the device owner's policy decides which signers may run | 19 | QEMU tests |
| [0041](../decisions/0041-immutable-packages.md) Installed programs are immutable packages named by their content hash | 19 | QEMU tests |
| [0042](../decisions/0042-updates.md) The system updates atomically as one image; third-party programs update package by package | 19, 20 | QEMU tests: updates and rollback |
| [0043](../decisions/0043-executable-memory-capability.md) Making memory executable at run time needs a capability | 5 | Host tests |
| [0044](../decisions/0044-system-image-is-os-shipped.md) Everything in the system image counts as shipped by the OS | 1 | Build check: the compiler wrapper covers the image |
| [0045](../decisions/0045-aarch64-with-first-user-program.md) AArch64 is brought up together with the first user-mode program | 8 | CI on both architectures from step 8 |
| [0046](../decisions/0046-scope-of-0008.md) Scope of 0008: the OS's own code, including platform-specific code | 1 | Build check: the architecture check, and review of platform code |
| [0047](../decisions/0047-kernel-mapping.md) The kernel stays mapped in every address space unless the CPU requires otherwise | 3 | Kernel tests, with the unmapped kernel forced |
| [0048](../decisions/0048-reserve-on-grant.md) Memory is reserved when it's granted; the system never promises more than it has | 12, 13 | QEMU tests |
| [0049](../decisions/0049-memory-fair-share.md) When memory runs short, it's shared fairly within each memory priority | 12 | QEMU tests |
| [0050](../decisions/0050-clients-pay.md) Clients pay for the state services keep for them | 12, 13, 17 | Attack test: a flooding client |
| [0051](../decisions/0051-kernel-clears-memory.md) The kernel clears memory before it's made into anything new | 4 | Host tests |
| [0052](../decisions/0052-user-mode-page-tables.md) Page tables are kernel objects that user mode builds | 5, 8, 10 | Host and QEMU tests |
| [0053](../decisions/0053-revoke-destroys-derived.md) Revoking memory destroys everything made from it | 4, 15 | Host tests |
| [0054](../decisions/0054-kernel-enforces-execute.md) The kernel enforces the right to make memory executable | 5, 19 | Host tests |
| [0055](../decisions/0055-kernel-maps-only-objects.md) The kernel maps its own memory and kernel objects, not all of physical memory | 3, 4 | Kernel tests |
| [0056](../decisions/0056-pager-chosen-at-spawn.md) A program's pager is chosen when it's spawned | 13 | QEMU tests |
| [0057](../decisions/0057-pager-fault-chain.md) A pager's own faults go to its pager, up a chain that ends at the root task | 10, 13 | QEMU tests |
| [0058](../decisions/0058-shared-and-handed-over.md) Programs share bulk data through regions that are shared or handed over | 9, 13 | QEMU tests |
| [0059](../decisions/0059-caller-allocates.md) Memory that crosses an interface is allocated by the caller, and its use moves only by handover | 13 | The bindings' tests |
| [0060](../decisions/0060-kernel-stack-per-core.md) The kernel has one stack per CPU core, and its worst-case depth is checked at build time | 2, 3 | Build check: the stack checker |
| [0061](../decisions/0061-no-randomization-yet.md) Nothing is randomized at first, and the design keeps randomization possible | 13 | QEMU tests: layouts repeat |
| [0062](../decisions/0062-memory-priority-capability.md) Memory priority is a capability of its own | 12 | QEMU tests |
| [0063](../decisions/0063-widest-addressing.md) The kernel turns on the widest virtual addressing the CPU offers | 3 | Kernel tests with 4- and 5-level paging |
| [0064](../decisions/0064-memory-belongs-to-payer.md) Memory belongs to the program it was granted to; lending, sharing and handover move only access | 12 | QEMU tests |
| [0065](../decisions/0065-lost-memory-is-an-error.md) Touching memory that was taken back is an error, not a crash | 13 | QEMU tests |
| [0066](../decisions/0066-kernel-upper-half.md) The kernel sits in the upper half of every address space, and programs in the lower half | 3, 5 | Kernel tests |
| [0067](../decisions/0067-object-window-offset.md) Kernel objects are mapped at their physical address plus a fixed offset | 3, 4 | Host tests |
| [0068](../decisions/0068-no-kernel-heap.md) The kernel has no heap | 2 | Build check: the kernel builds on core alone |
| [0069](../decisions/0069-all-page-sizes.md) The kernel supports every page size the hardware offers | 5, 8 | Host and QEMU tests |
| [0070](../decisions/0070-position-independent-programs.md) Programs are built position-independent | 13 | Build check: programs are position-independent |
| [0071](../decisions/0071-three-memory-programs.md) The root task, the memory service and the default pager are three separate programs | 10 | QEMU tests |
| [0072](../decisions/0072-registers-only.md) System calls and IPC carry data only in registers; the kernel never reads program data | 6, 9 | Review, and the guards fault on any access ([0075](../decisions/0075-user-guards-always-on.md)) |
| [0073](../decisions/0073-kernel-layout-in-implementation.md) The kernel's internal layout is chosen during implementation | 3 | Not testable: it leaves a choice to implementation |
| [0074](../decisions/0074-low-64k-unmapped.md) The lowest 64 KiB of every program's half stays unmapped | 13 | QEMU tests |
| [0075](../decisions/0075-user-guards-always-on.md) The kernel's guards against touching or running program memory are always on | 3, 8 | Kernel tests |
| [0076](../decisions/0076-memory-notice-deadline.md) Programs over their fair share get a notice, then a deadline | 12 | QEMU tests |
| [0077](../decisions/0077-swap-in-default-pager.md) Swap is a feature of the default pager; it comes later and is encrypted | Later | Not testable: later work |
| [0078](../decisions/0078-low-addresses-on-request.md) Programs that need low addresses ask for them | 13 | QEMU tests |
| [0079](../decisions/0079-pointer-tagging-off.md) Hardware pointer tagging stays off for now | 3, 8 | Kernel tests: tagging stays off |
| [0080](../decisions/0080-predictor-flush-both-ways.md) Branch predictors are flushed when switching to or away from a program that asks | 5 | Kernel tests |
| [0081](../decisions/0081-pager-per-thread.md) Page faults go to the faulting thread's pager | 9 | Host and QEMU tests |
| [0082](../decisions/0082-corrections.md) Corrections to 0003, 0006, 0008, 0014 and 0077 | — | Not testable: corrections |
| [0083](../decisions/0083-program-is-address-space.md) To the kernel, a program is an address space and the threads bound to it | 5 | Host tests |
| [0084](../decisions/0084-shares-belong-to-accounts.md) Fair shares of CPU time and memory belong to accounts | 11, 12 | QEMU tests |
| [0085](../decisions/0085-clients-pay-cpu-time.md) Clients pay for the CPU time services spend on them | 9, 11 | Attack test: a flooding client |
| [0086](../decisions/0086-priorities-per-program-shares-per-account.md) Priorities stay per program, and each priority is shared by account | 11 | QEMU tests |
| [0087](../decisions/0087-thread-priorities.md) A thread runs at any CPU priority up to its program's | 5 | Host tests |
| [0088](../decisions/0088-user-mode-scheduler.md) Scheduling policy runs in user mode | 11 | QEMU tests |
| [0089](../decisions/0089-interrupts-off-in-kernel.md) The kernel runs with interrupts off between its interruption points | 3, 4 | The simulator's work counts, and timing on the laptop |
| [0090](../decisions/0090-priorities-in-kernel.md) Strict priorities stay in the kernel, and the scheduler orders threads within each priority | 5 | Host tests |
| [0091](../decisions/0091-direct-switch-on-calls.md) Calls and replies switch directly between client and service | 9 | Host tests |
| [0092](../decisions/0092-scheduler-keeps-time.md) The scheduler keeps the CPU time books, from the kernel's reports | 9, 11 | Host and QEMU tests |
| [0093](../decisions/0093-scheduler-per-core.md) The scheduler has a thread on each core, which decides for that core | 11, 15 | QEMU tests |
| [0094](../decisions/0094-one-deadline-per-core.md) Each core has one timer deadline, set by its scheduler thread | 3, 5, 11 | Host and QEMU tests |
| [0095](../decisions/0095-one-kernel-lock.md) One lock guards the kernel on multicore | 3, 15 | Simulator tests |
| [0096](../decisions/0096-object-window-lazy-unmap.md) The object window keeps memory mapped until it becomes a program's pages | 4 | Host tests |
| [0097](../decisions/0097-user-mode-starts-cores.md) User mode finds the CPUs, and the root task asks the kernel to start each one | 15 | QEMU tests |
| [0098](../decisions/0098-core-sharing-capability.md) Programs share a core only if both hold the same core-sharing capability | 15 | QEMU tests |
| [0099](../decisions/0099-kernel-beside-sibling.md) The kernel doesn't pause a sibling hardware thread when it runs | 15 | Review |
| [0100](../decisions/0100-cpu-flaw-mitigations.md) The kernel applies each CPU's documented mitigations for speculative-execution flaws | 3, 8, 15 | Kernel tests on QEMU's CPU models |
| [0101](../decisions/0101-eager-fpu-state.md) Floating-point and vector registers are saved and restored on every switch | 5, 8 | Host and kernel tests |
| [0102](../decisions/0102-exception-handler.md) Each thread names an exception handler besides its pager | 9 | QEMU tests |
| [0103](../decisions/0103-fault-without-handler.md) A fault with nowhere to go suspends the thread, and in the root task halts the system | 9 | QEMU tests |
| [0104](../decisions/0104-supervisor.md) Every program has a supervisor, named at spawn, that learns how it ended | 10, 19 | QEMU tests |
| [0105](../decisions/0105-supervisor-reclaims.md) A failed program's supervisor has its memory taken back, and decides on a restart | 10, 12 | QEMU tests |
| [0106](../decisions/0106-pager-passes-on-faults.md) A pager passes faults it won't resolve to the thread's exception handler | 9, 13 | QEMU tests |
| [0107](../decisions/0107-loader-writes-startup-data.md) The loader writes a new program's startup information into its memory | 13 | QEMU tests |
| [0108](../decisions/0108-startup-information.md) What a new program receives when it's spawned | 13 | QEMU tests |
| [0109](../decisions/0109-panic-strategy.md) Third-party programs unwind on panic by default, and the OS's own services abort | 10, 14 | QEMU tests: services abort, programs unwind |
| [0110](../decisions/0110-kernel-panic.md) A kernel panic is logged, then the machine halts or reboots by a boot setting | 2 | Kernel tests |
| [0111](../decisions/0111-resource-accounting.md) Each service reports the resource use it already tracks | 11, 12, 21 | QEMU tests |
| [0112](../decisions/0112-time-service.md) Programs read the time from a time service | 14 | QEMU tests |
| [0113](../decisions/0113-counter-capability.md) Only programs holding a capability may read the CPU's time counter | 3, 5, 14 | Kernel and QEMU tests |
| [0114](../decisions/0114-scheduler-priority.md) Each core's scheduler thread runs above every priority it schedules | 11 | QEMU tests |
| [0115](../decisions/0115-root-task-fault-boot-setting.md) A fault in the root task is handled like a kernel panic | 8 | QEMU tests |
| [0116](../decisions/0116-name-thisos.md) The OS is named thisOS | — | Not testable: a name |
| [0117](../decisions/0117-system-library.md) Programs reach the kernel only through the system library | 6, 8, 13 | QEMU tests: calls from elsewhere are refused |
| [0118](../decisions/0118-calls-and-notifications.md) IPC is synchronous calls through endpoints, with notifications for events | 9 | Host tests |
| [0119](../decisions/0119-message-eight-words.md) One IPC message carries up to 8 words | 9 | Host tests |
| [0120](../decisions/0120-reply-objects.md) Services reply through reply objects | 9 | Host tests |
| [0121](../decisions/0121-callers-help-busy-services.md) A caller that finds its service busy lends it its priority and time | 9 | Attack test: inversion through a busy service |
| [0122](../decisions/0122-bound-notifications.md) A notification can be bound to a thread | 9 | Host tests |
| [0123](../decisions/0123-capabilities-in-messages.md) A message carries up to four capabilities, each copied or moved | 9 | Host tests |
| [0124](../decisions/0124-large-calls-split.md) Kernel calls whose data doesn't fit in registers are split, and the system library joins them | 6, 9 | QEMU tests |
| [0125](../decisions/0125-sysret-guard.md) The kernel never returns with `sysret` to an address outside the program half | 6 | Kernel tests |
| [0126](../decisions/0126-capability-space-radix-table.md) A capability space is a radix table, and capabilities are named by index | 4 | Host tests |
| [0127](../decisions/0127-capability-space-per-program.md) Each program has exactly one capability space | 4 | Host tests |
| [0128](../decisions/0128-rights-and-badges.md) Capabilities carry a few rights, and endpoint and notification capabilities carry a badge | 4 | Host tests |
| [0129](../decisions/0129-revoke-any-capability.md) Revoking any capability destroys every copy made from it | 4 | Host tests |
| [0130](../decisions/0130-handlers-changeable.md) A thread's pager and exception handler can be changed through its control right | 9 | Host tests |
| [0131](../decisions/0131-namespace-path-prefixes.md) A namespace maps path prefixes to capabilities | 13 | QEMU tests |
| [0132](../decisions/0132-events-notify-then-call.md) An event is a notification, followed by a call for its details | 9, 12 | QEMU tests |
| [0133](../decisions/0133-recovery-point-in-register.md) Accessors keep their recovery point in a register | 9, 13 | QEMU tests |
| [0134](../decisions/0134-scheduler-interface.md) Threads run once the scheduler takes them on, and the kernel reports to it in registers | 5, 9, 11 | Host and QEMU tests |
| [0135](../decisions/0135-core-sharing-from-scheduler.md) The scheduler issues core-sharing capabilities | 11, 15 | QEMU tests |
| [0136](../decisions/0136-predictor-flush-flag.md) A program asks for predictor flushes with a flag on its address space | 5, 13 | Host and QEMU tests |
| [0137](../decisions/0137-memory-capability-per-program.md) Each program has its own capability to the memory service, which names it as payer | 12 | QEMU tests |
| [0138](../decisions/0138-pager-keeps-reservations.md) Pagers keep a program's address-space reservations | 13 | QEMU tests |
| [0139](../decisions/0139-results-fill-and-report.md) A result of unknown size fills what the caller provides, and reports what remains | 13 | The bindings' tests |
| [0140](../decisions/0140-accounts-capability-pair.md) An account is a pair of capabilities, one from the scheduler and one from the memory service | 11, 12, 21 | QEMU tests |
| [0141](../decisions/0141-identity-capability.md) The login service gives each session an identity capability | 21 | QEMU tests |
| [0142](../decisions/0142-own-idl.md) Interfaces are defined in our own IDL, modeled on FIDL | 6 | The IDL compiler's tests |
| [0143](../decisions/0143-add-only-versioning.md) Interfaces only grow, and a tool checks every change | 6, 23 | Build check: the compatibility checker |
| [0144](../decisions/0144-c-calls-and-what-crosses.md) The system library uses the C calling convention, and only data, capabilities and regions cross between programs | 6 | The IDL compiler's tests, and review |
| [0145](../decisions/0145-std-over-native-api.md) std is a layer over the native API | 10 | Review |
| [0146](../decisions/0146-startup-data-block.md) Startup data is a versioned block defined in the IDL | 6, 13 | QEMU tests |
| [0147](../decisions/0147-elf-declarations-only.md) A program's ELF sections carry only its declarations | 13 | QEMU tests |
| [0148](../decisions/0148-freeze-covers-rules.md) The freeze covers the interface design, and each interface's version 1 is fixed when it's released | 23 | Build check: the compatibility checker |
| [0149](../decisions/0149-bootloader-crate-uefi.md) The kernel is booted by the `bootloader` crate, through UEFI only | 2 | QEMU tests |
| [0150](../decisions/0150-bootloader-aarch64.md) AArch64 support is added to the `bootloader` crate | 7 | QEMU tests on AArch64 |
| [0151](../decisions/0151-boot-info-from-crate.md) The kernel starts from the crate's boot information | 2, 3, 8 | Kernel tests |
| [0152](../decisions/0152-kernel-log-and-reset.md) The kernel writes its log to memory and a serial port, and resets the machine itself | 2, 3 | Kernel tests |
| [0153](../decisions/0153-build-unpacks-ramdisk.md) The build unpacks the root task and the system library into the ramdisk | 2, 8, 10 | QEMU tests |
| [0154](../decisions/0154-library-entry-table.md) Programs call the system library through a table of entry points | 6 | QEMU tests |
| [0155](../decisions/0155-ramdisk-reaches-storage.md) The ramdisk holds the programs needed to reach storage | 13, 17, 19 | Image tests |
| [0156](../decisions/0156-boot-loader-checks-image.md) The boot loader checks the kernel and the ramdisk against the system image's signature | 20 | Attack test: a tampered kernel or ramdisk |
| [0157](../decisions/0157-boot-settings-in-ramdisk.md) Boot settings live in the ramdisk's header | 2 | Kernel tests |
| [0158](../decisions/0158-image-try-count.md) A new system image is tried a limited number of times, then the boot loader falls back | 20 | QEMU tests: fallback |
| [0159](../decisions/0159-root-task-boot-information.md) The root task receives the usual startup data, with a block of boot information | 6, 8, 10 | QEMU tests |
| [0160](../decisions/0160-root-task-starts-four.md) The root task starts four programs itself, and the rest from a boot manifest | 10, 13 | QEMU tests |
| [0161](../decisions/0161-root-task-before-scheduler.md) The root task runs alone until it starts the scheduler | 8, 11 | QEMU tests |
| [0162](../decisions/0162-scheduler-above-priorities.md) A core's scheduler thread runs above every priority, by a rule in the kernel | 5, 11 | Host tests |
| [0163](../decisions/0163-services-hand-first-capabilities.md) A service hands the first capabilities it issues to the program that started it | 11, 12 | QEMU tests |
| [0164](../decisions/0164-root-task-memory-pool.md) The root task keeps back a pool of memory for the programs it pages | 10 | QEMU tests |
| [0165](../decisions/0165-core-objects.md) Each core is a kernel object, named by its capability | 9, 15 | QEMU tests |
| [0166](../decisions/0166-core-revoke-offline.md) Revoking a running core's memory takes the core offline | 15 | QEMU tests |
| [0167](../decisions/0167-interrupts-as-notifications.md) Interrupts reach drivers as notifications | 9, 16 | QEMU tests |
| [0168](../decisions/0168-device-memory-and-ports.md) Device memory and I/O ports are capabilities of their own | 5, 16 | Host and QEMU tests |
| [0169](../decisions/0169-iommu-service.md) A user-mode IOMMU service keeps the devices' I/O page tables | 16 | Attack test: a device writing outside its mappings |
| [0170](../decisions/0170-dma-quarantine.md) Memory mapped for a device is held back after a revoke, until the IOMMU service releases it | 16 | QEMU tests |
| [0171](../decisions/0171-no-iommu-trusted-drivers.md) Without an IOMMU, the drivers of DMA-capable devices are trusted | 15, 16 | Image tests without an IOMMU |
| [0172](../decisions/0172-device-manager.md) A device manager starts each driver with exactly its device's capabilities | 16 | QEMU tests |
| [0173](../decisions/0173-time-service-entry.md) Programs reach the time service through the namespace entry `svc/time` | 14, 16 | QEMU tests |
| [0174](../decisions/0174-random-service-and-seed.md) Random numbers come from a random service, and every program starts with a seed | 14 | QEMU tests |
| [0175](../decisions/0175-rust-target-names.md) The Rust targets are `x86_64-unknown-thisos` and `aarch64-unknown-thisos`, with no target family | 10 | Build check: the target specs |
| [0176](../decisions/0176-paths-and-current-directory.md) Paths that start with `/` resolve through the namespace, and others through a current directory | 19 | Std tests |
| [0177](../decisions/0177-locks-on-parking.md) std's locks are built on parking, and parking on notifications | 5, 14 | Std tests |
| [0178](../decisions/0178-native-api-crate.md) The native API is a separate crate | 19, 23 | QEMU tests |
| [0179](../decisions/0179-command-copies-namespace.md) std's `Command` gives a child a copy of the spawner's namespace by default | 13, 19 | Std and QEMU tests |
| [0180](../decisions/0180-streams-and-exit-status.md) Standard streams use a byte-stream protocol, and a program ends with a code or an abnormal end | 14, 19 | Std tests |
| [0181](../decisions/0181-network-service.md) `std::net` uses a network service through the namespace | 22 | Std and QEMU tests |
| [0182](../decisions/0182-all-of-std.md) All of std is supported, in stages, and allocation failure aborts | 10, 14, 19, 22 | Std tests |
| [0183](../decisions/0183-access-on-directories.md) Stored access information lives on directories, as grants to stable user IDs | 17, 21 | QEMU tests |
| [0184](../decisions/0184-shared-file-pages.md) Read-only file pages are shared from the service's cache, which pays for them | 17, 19 | QEMU tests |
| [0185](../decisions/0185-swap-partition.md) Swap goes to a partition, through the block-storage service | 17 | QEMU tests of the partition layout; swap itself is later work |
| [0186](../decisions/0186-non-rust-route.md) Non-Rust software runs on WebAssembly first, then natively on the native API | Later | Not testable: later work |
| [0187](../decisions/0187-pinned-stable-toolchain.md) The toolchain is a pinned stable Rust release, and std's platform layer is a patch over its library | 1, 10, 23 | Build check: the pinned toolchain |
| [0188](../decisions/0188-upgrade-every-second-release.md) The toolchain is upgraded every second stable release | Recurring | Review of each upgrade |
| [0189](../decisions/0189-kernel-none-targets.md) The kernel builds for the built-in `-none` targets | 2 | Build check: the kernel's targets |
| [0190](../decisions/0190-cargo-and-build-tool.md) Cargo builds thisOS, driven by a build tool written in Rust | 1, 2 | The build tool's tests |
| [0191](../decisions/0191-audited-list-and-wrapper.md) A list of audited crates, enforced by a compiler wrapper, keeps unsafe code in the audited set | 1 | Build check: the compiler wrapper |
| [0192](../decisions/0192-build-scripts-reviewed.md) Build scripts and procedural macros run only in reviewed crates, and never link native code | 1 | Build check: the compiler wrapper and the sandbox |
| [0193](../decisions/0193-vendored-reviewed-crates.md) Third-party crates in the OS's own code are vendored and reviewed | 1 | Build check: cargo-vet and cargo-deny |
| [0194](../decisions/0194-reproducible-builds.md) The system image and the boot loader build reproducibly | 2, 24 | The reproducibility check |
| [0195](../decisions/0195-bindings-safe-output.md) The build tool generates the bindings, and the generated code contains no unsafe code | 6, 13, 23 | Build check: the compiler wrapper on generated code |
| [0196](../decisions/0196-released-interface-copies.md) Released interface versions are kept in the repository, and every change is checked against them | 6, 23 | Build check: the compatibility checker |
| [0197](../decisions/0197-architecture-check.md) Crates are marked by architecture, and the build rejects architecture-specific code in generic crates | 1 | Build check: the architecture check |
| [0198](../decisions/0198-stack-depth-checker.md) A checker on the linked kernel bounds its stack depth and rejects recursion | 2 | Build check: the stack checker |
| [0199](../decisions/0199-license.md) thisOS is licensed under MIT OR Apache-2.0 | 1 | Build check: cargo-deny's license check |
| [0200](../decisions/0200-contributions.md) Contributions come in under the project's license, with no sign-off | 1 | Not testable: a contribution policy |
| [0201](../decisions/0201-changes-after-freeze.md) After the freeze, a decision changes only through a new record that cites what building found | — | Not testable: a process |
| [0202](../decisions/0202-offline-attackers.md) Attackers who get the machine while it's off are in scope | 20, later | Attack tests: tampered images and packages |
| [0203](../decisions/0203-package-format.md) A package is a signed manifest and files named by SHA-256 Merkle roots | 19 | Attack test: a tampered block |
| [0204](../decisions/0204-hybrid-signatures.md) Signatures pair Ed25519 with ML-DSA-65 | 19, 20 | QEMU tests |
| [0205](../decisions/0205-root-and-working-keys.md) An offline root key names the working keys, and the boot loader carries only the root | 20, 24 | Attack test: a revoked working key |
| [0206](../decisions/0206-owner-approves-signers.md) A program runs if the system image's signer or a signer the owner approved signed it | 19, 21, 23 | QEMU tests |
| [0207](../decisions/0207-install-and-start-checks.md) The package store checks packages at install and after each boot, and the loader checks the policy at each start | 19, 23 | Attack test: a tampered manifest |
| [0208](../decisions/0208-boot-loader-beside-old.md) A new boot loader is installed beside the previous one, which the firmware falls back to | 20 | QEMU tests |
| [0209](../decisions/0209-no-rollback-protection.md) Older signed images still boot, and rollback protection is later work | 2, 20 | QEMU tests: older signed images boot |
| [0210](../decisions/0210-std-version-declared.md) Programs declare the std they were built with, and the owner can refuse flawed ones | 19, 23 | QEMU tests |
| [0211](../decisions/0211-relative-relocations.md) Programs carry only relative relocations | 13 | QEMU tests: other relocations are refused |
| [0212](../decisions/0212-low-image-declaration.md) A program can declare the highest address for its image | 13 | QEMU tests |
| [0213](../decisions/0213-power-capability.md) A power capability lets its holder reboot, power off or fail the machine | 3, 8, 15 | QEMU tests |
| [0214](../decisions/0214-interrupt-remapping.md) Devices raise only the interrupts they were given, through interrupt remapping | 16 | Attack test: a device's stray MSI |
| [0215](../decisions/0215-pci-bus-driver-config.md) The PCI bus driver keeps configuration space, and mediates drivers' access to it | 16 | Attack test: refused configuration writes |
| [0216](../decisions/0216-essential-service-failure.md) A failure of an essential service is handled like a fault in the root task | 10 | QEMU tests |
| [0217](../decisions/0217-window-flush-batched.md) Object-window flushes are batched, at most at a set rate | 4, 13, 15 | Attack test: a window-flush loop |
| [0218](../decisions/0218-lock-yield.md) Long kernel operations also yield the kernel lock at their interruption points | 4, 15 | Attack test: a revoke that would hold the lock |
| [0219](../decisions/0219-wakeup-preemption.md) A wakeup enters the scheduler only if the woken thread may run before the running one | 9 | Attack test: a wakeup storm |
| [0220](../decisions/0220-endpoint-priority-order.md) Callers waiting on an endpoint are served by priority, then arrival | 9 | Host tests |
| [0221](../decisions/0221-corrections.md) Corrections to 0003 and 0153 | — | Not testable: corrections |
| [0222](../decisions/0222-roadmap-to-first-release.md) The roadmap runs from an empty repository to a first release of the whole design | — | Not testable: the roadmap's scope |
| [0223](../decisions/0223-qemu-and-laptop-checkpoints.md) thisOS is built and tested in QEMU, with a checkpoint on the development laptop at the end of each phase | All | Not testable: where testing runs |
| [0224](../decisions/0224-laptop-usb-storage.md) On the laptop, thisOS keeps its storage on a USB disk, and never opens the internal disk | 17, 18 | Image tests: laptop images start no driver for the internal disk |
| [0225](../decisions/0225-framebuffer-log-at-failures.md) The kernel shows its log on the framebuffer when it fails, and while bring-up images start | 2 | Kernel tests: forced failures draw the log |
| [0226](../decisions/0226-open-topics-at-their-step.md) Topics no record decides are designed at the step that needs them | — | Not testable: a process |
| [0227](../decisions/0227-steps-and-phases.md) The roadmap has 24 steps in 7 phases, each after the steps it needs | — | Not testable: the plan itself |
| [0228](../decisions/0228-step-start-and-done.md) A step starts once the steps it needs are done, and is done when it's tested, reviewed and mapped | All | The coverage map's check, and review |
| [0229](../decisions/0229-std-stages.md) std comes in four stages, each done when upstream's tests for it pass | 10, 14, 19, 22 | Std tests |
| [0230](../decisions/0230-interfaces-released-with-sdk.md) Interfaces that third-party programs reach are released with the SDK | 23 | Build check: the compatibility checker |
| [0231](../decisions/0231-repository-layout.md) One repository holds thisOS, with a Cargo workspace for each kind of target | 1 | Not testable: a layout |
| [0232](../decisions/0232-traits-and-specific-crates.md) Generic crates state their hardware needs as traits, and specific crates implement them | 1 | Build check: the dependency check |
| [0233](../decisions/0233-simulated-machine.md) The kernel's generic part also runs on the host, on a simulated machine | 4 | The simulator's own tests |
| [0234](../decisions/0234-tests-cite-records.md) Tests name the records they check, and the build tool maps every decision to its tests | 1 | Build check: the coverage map |
| [0235](../decisions/0235-kinds-of-tests.md) Every change runs host, QEMU, check and attack tests, and Miri runs on a test-only nightly | All | Not testable: it defines the tests |
| [0236](../decisions/0236-program-output-second-serial.md) Programs write to a second serial port until the log service takes it, and test runs end through the power capability | 8, 14 | QEMU tests |
| [0237](../decisions/0237-public-repository-and-ci.md) thisOS is public on GitHub, where CI runs the build tool's `ci` command for every change | 1 | Not testable: where CI runs |
| [0238](../decisions/0238-pushes-and-pull-requests.md) Maintainers push to main after a local `ci` run, and others send pull requests | 1 | Not testable: a process |
| [0239](../decisions/0239-security-reports.md) Security problems are reported privately, through GitHub | 1 | Not testable: a process |
| [0240](../decisions/0240-timing-on-laptop.md) Benchmarks and time budgets run on the development laptop | 15 | Not testable: where timing runs |
| [0241](../decisions/0241-reproducibility-checks.md) The image and the boot loader are built twice in CI, and rebuilt elsewhere before signing | 2, 24 | The reproducibility check |
| [0242](../decisions/0242-review-tiers.md) Changes are reviewed in three tiers, by role | All | Not testable: a review process |
| [0243](../decisions/0243-audit-records-and-crate-reviews.md) Each audited crate has a record beside the list, and cargo-vet records crate reviews | 1 | Build check: every listed crate has its record |
| [0244](../decisions/0244-imported-crate-reviews.md) Crate reviews may be imported from named organizations, outside the audited set | 1 | Build check: cargo-vet |
| [0245](../decisions/0245-coverage-and-fuzzing.md) Host tests cover every line of the generic kernel, and system calls are fuzzed from step 4 | 4, 9 | Coverage measured in CI |
| [0246](../decisions/0246-interruption-stretch-limit.md) Work between interruption points is limited in code, and no stretch may take more than 20 µs | 4, 15 | The simulator's work counts, and timing on the laptop |
| [0247](../decisions/0247-kernel-size-ratchet.md) A record rule and a size ratchet keep the kernel small | 1, 9 | Build check: the size count |
| [0248](../decisions/0248-coding-and-doc-standards.md) The code follows one set of coding and documentation standards | 1 | Build check: the lints |
| [0249](../decisions/0249-estimates-and-checkpoints.md) Steps are estimated by the code they add, and calibrated at each phase's checkpoint | — | Not testable: a process |
| [0250](../decisions/0250-plan-changes-after-freeze.md) After the freeze, steps change only through a record that cites what building found | — | Not testable: a process |
| [0251](../decisions/0251-later-work.md) Later work stays unscheduled, and the first release states the gaps it leaves | 24 | Not testable: a process |
| [0252](../decisions/0252-time-service-starts-early.md) The time service starts early, and serves wall-clock time once the clock driver reports it | 14, 16 | QEMU tests |
| [0253](../decisions/0253-ramdisk-log-and-clocks.md) The ramdisk also holds the log service and the clock drivers | 14, 16 | Image tests |
| [0254](../decisions/0254-corrections.md) Corrections to 0045 | — | Not testable: corrections |
| [0255](../decisions/0255-freeze.md) The design and the roadmap freeze with Day 8's commit, on 2026-10-19 | — | Not testable: a process |

## The freeze

Day 8's commit, on 2026-10-19, froze the design and the roadmap ([0255](../decisions/0255-freeze.md)). Every design document's status is Frozen. New records from here on either follow [0201](../decisions/0201-changes-after-freeze.md) and [0250](../decisions/0250-plan-changes-after-freeze.md), or settle open topics and later work ([0226](../decisions/0226-open-topics-at-their-step.md), [0251](../decisions/0251-later-work.md)).

## Open questions

This document has none left. The open topics above get designed at their steps.
