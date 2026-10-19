# 0249. Steps are estimated by the code they add, and calibrated at each phase's checkpoint

- **Status:** Accepted, amended by [0264](0264-performance-targets.md)
- **Date:** 2026-10-19

## Context

Day 8 asks how long each step is expected to take, and where progress is checked against the plan ([08-roadmap](../design/08-roadmap.md)). Every change gets a review, line by line in tier 1 ([0242](0242-review-tiers.md)), so a step's time depends on how much code it adds, and in which tier. Nothing has been measured yet. Each phase ends with a checkpoint on the laptop ([0223](0223-qemu-and-laptop-checkpoints.md)).

## Options

1. **By size, calibrated.**
2. **Weeks per step,** set now.
3. **No estimates.**

## Decision

Option 1.

- Each step has an estimate of the code it adds, by tier, and of the review time that needs ([08-roadmap](../design/08-roadmap.md)).
- Calendar dates come from the review pace measured on earlier steps, from checkpoint 1 on.
- At each phase's checkpoint:
  - actual sizes and times are compared with the estimates;
  - the remaining steps are re-estimated;
  - the laptop runs that phase's image ([0223](0223-qemu-and-laptop-checkpoints.md));
  - the kernel's size counts are reviewed ([0247](0247-kernel-size-ratchet.md));
  - the results are recorded in [08-roadmap](../design/08-roadmap.md).

## Consequences

- **Dates rest on measured pace,** not on guesses made before any code exists.
- **The first estimates are rough** until checkpoint 1.
- **A slipping step shows up by the next checkpoint** at the latest.
- **Day 8's question on estimates and checkpoints is answered.**
