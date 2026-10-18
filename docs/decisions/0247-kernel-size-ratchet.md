# 0247. A record rule and a size ratchet keep the kernel small

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The kernel is as small as possible, and this is enforced heavily ([0019](0019-kernel-minimality.md)). 0019 left the means open: review rules, a size budget, or both. The final review closed the list of what the kernel contains, and adding to it after the freeze takes a record ([01-objective](../design/01-objective.md), [0201](0201-changes-after-freeze.md)). All of the kernel's mechanisms are in by the end of step 9 ([0227](0227-steps-and-phases.md)).

## Options

1. **A record rule, and a ratchet.**
2. **A record rule, and a fixed budget** set now.
3. **A record rule only.**

## Decision

Option 1.

- A new kernel object type, system call, right or field that no record names needs a record ([0201](0201-changes-after-freeze.md)).
- From step 1, the build tool counts the kernel's lines of code, without tests or comments, separately for the generic kernel, each architecture and each platform.
- At the end of step 9, the counts become a budget, kept in the repository.
- A change that grows the kernel past its budget raises the budget in the same change, with the reason. It's reviewed as a tier 1 change ([0242](0242-review-tiers.md)).
- Each checkpoint reviews how the counts have moved ([0249](0249-estimates-and-checkpoints.md)).

## Consequences

- **The kernel can't grow without someone deciding that it should.**
- **The budget comes from the kernel as built,** not from a guess made before any code exists.
- **Every increase has its reason on record,** in the change that made it.
- **0019's question on enforcement is answered.**
