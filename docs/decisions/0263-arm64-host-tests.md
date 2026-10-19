# 0263. Host tests also run on arm64, and lock-free code is model-checked

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

AArch64 is brought up early so that its weaker memory ordering finds concurrency bugs that x86_64 hides ([0009](0009-x86-64-first.md), [0045](0045-aarch64-with-first-user-program.md), [0227](0227-steps-and-phases.md)). It runs only under emulation, locally and in CI ([0223](0223-qemu-and-laptop-checkpoints.md), [0237](0237-public-repository-and-ci.md)). QEMU lets an emulated guest see stronger ordering than its architecture's, never weaker, so on an x86_64 host the guest sees close to x86's ordering. The audit found this in QEMU's documentation ([0256](0256-audit-findings-as-evidence.md)).

GitHub's arm64 runners serve public repositories ([0237](0237-public-repository-and-ci.md)). Host tests run the generic kernel on the simulated machine, and the audited set's code that runs on the host ([0233](0233-simulated-machine.md), [0235](0235-kinds-of-tests.md)). loom explores the interleavings and orderings of concurrent Rust code, and Miri emulates weak memory for atomics ([0235](0235-kinds-of-tests.md)).

## Options

1. **arm64 runners, with loom and Miri.**
2. **The same, plus a real AArch64 board** at each checkpoint from checkpoint 3.
3. **loom and Miri only.**
4. **The gap accepted,** with 0227 corrected.

## Decision

Option 1.

- CI also runs the host tests natively on GitHub's arm64 runners, from step 1.
- Lock-free code in the kernel, the audited set and std's patch is checked with loom.
- Miri keeps its weak-memory emulation on.
- QEMU tests on AArch64 stay as they are. They check that the code runs, not its memory ordering.
- Step 1 designs this with the CI workflows ([0226](0226-open-topics-at-their-step.md)).
- This amends 0235, whose host tests also run on arm64 and gain loom, and 0237, whose CI gains the arm64 runners.

## Consequences

- **Generic code meets real weak ordering** on every change.
- **Code that runs only in QEMU never meets it on real hardware.** Its lock-free parts, such as the kernel lock ([0095](0095-one-kernel-lock.md)), are model-checked with loom instead.
- **What 0009, 0045 and 0227 say about finding ordering bugs** is corrected in [0271](0271-corrections.md).
- **A local `ci` run can't run the arm64 tests** on an x86_64 machine. They run in hosted CI after the push, and a failure there is handled as 0238 says ([0238](0238-pushes-and-pull-requests.md)). On x86_64, the local run still predicts everything else ([0237](0237-public-repository-and-ci.md)).
- **Miri still runs on its test-only nightly** ([0235](0235-kinds-of-tests.md)).
- **Step 1 is affected** ([08-roadmap](../design/08-roadmap.md)).
