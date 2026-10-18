# 0228. A step starts once the steps it needs are done, and is done when it's tested, reviewed and mapped

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The kernel has strict review, high test coverage and fuzzing ([0018](0018-kernel-assurance.md)). AArch64 joins at the first user-mode program, and CI covers both architectures from then on ([0045](0045-aarch64-with-first-user-program.md)). Each step names the steps it needs ([0227](0227-steps-and-phases.md)), and tests name the records they check ([0234](0234-tests-cite-records.md)).

## Options

1. **By dependency.** A step starts once the steps it needs are done, so independent steps can run side by side.
2. **One at a time.** Each step starts only when the one before it is done.
3. **Done by tests,** with reviews catching up at each checkpoint.

## Decision

Option 1.

- Each step names the steps it needs ([08-roadmap](../design/08-roadmap.md)), and starts once they're done.
- A step is done when:
  - its tests pass on every architecture it supports: x86_64 for steps 1 to 6, AArch64 for step 7, and both from step 8 on;
  - CI passes ([0237](0237-public-repository-and-ci.md));
  - its review is complete ([0242](0242-review-tiers.md));
  - every decision it implements has its tests in the coverage map ([0234](0234-tests-cite-records.md));
  - its documents are up to date, and [08-roadmap](../design/08-roadmap.md) marks it done.

## Consequences

- **Nothing builds on unreviewed work.**
- **Independent steps can overlap,** such as the AArch64 boot loader beside the kernel's steps, and USB beside storage.
- **CI covers both architectures from step 8,** as 0045 requires.
- **Day 8's question on when a step is done is answered.**
