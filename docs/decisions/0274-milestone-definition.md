# 0274. A milestone is a working image that's reviewed, passes a security pass and is tagged

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

The roadmap reaches the whole design through milestones ([0273](0273-whole-design-in-milestones.md)). Each phase ends with a checkpoint on the laptop ([0223](0223-qemu-and-laptop-checkpoints.md)), which calibrates the estimates ([0249](0249-estimates-and-checkpoints.md)), and from checkpoint 3 on sets performance targets ([0264](0264-performance-targets.md)). A step is done when its tests pass, CI passes, its review is finished and its records have their tests ([0228](0228-step-start-and-done.md)).

## Options

1. **A working image, reviewed, passing a security pass, and tagged.**
2. **The phases' checkpoints, tagged,** with reviews only.
3. **A published release** at each milestone, with a signed image.

## Decision

Option 1. A milestone is done when all of this holds:

- its image boots in QEMU, on each architecture its steps support, with and without an IOMMU once one is built ([0223](0223-qemu-and-laptop-checkpoints.md));
- the laptop boots it from a USB stick, and runs it as far as its devices allow ([0223](0223-qemu-and-laptop-checkpoints.md), [0224](0224-laptop-usb-storage.md));
- every step it holds is done ([0228](0228-step-start-and-done.md));
- CI passes on the commit it tags ([0237](0237-public-repository-and-ci.md));
- its security pass is written, and its gaps are settled ([0276](0276-milestone-security-pass.md));
- its results are in the progress log and in 08-roadmap: the review hours it took, its laptop run, and its performance measurements ([0277](0277-sized-work-no-dates.md), [0279](0279-progress-log.md)).

Its commit is then tagged `m1` to `m8` ([0291](0291-tags-protected.md)).

- The checkpoints of 0223, 0249 and 0264 become the milestones' laptop runs, calibrations and targets.
- This amends 0223, 0249 and 0264, whose checkpoints are now the milestones.

## Consequences

- **Every milestone image is a system that someone can boot,** and the laptop sees each one.
- **Performance targets are set at m3, m4 and m5,** where the checkpoints after steps 9, 14 and 16 set them ([0264](0264-performance-targets.md)).
