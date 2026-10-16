# 0135. The scheduler issues core-sharing capabilities

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Two programs run on sibling hardware threads only if both hold the same core-sharing capability ([0098](0098-core-sharing-capability.md)). Pairing them is the scheduler's policy ([0034](0034-smt-trusted-pairs.md), [0093](0093-scheduler-per-core.md)). 0098 left open how the capability is made and checked.

## Options

1. **Issued by the scheduler.** A badged capability to the scheduler, with one badge for each sharing group.
2. **A kernel object,** which the kernel checks whenever the scheduler places threads side by side.

## Decision

Option 1.

- A core-sharing capability is a badged capability to the scheduler ([0128](0128-rights-and-badges.md)). Each badge is one sharing group.
- Any program can ask the scheduler for a new group, and pass it to the programs it trusts.
- A program presents it when it enrols its threads ([0134](0134-scheduler-interface.md)). The scheduler pairs only threads whose programs presented the same group.

## Consequences

- **The kernel isn't involved,** and gains no object type ([0019](0019-kernel-minimality.md)).
- **The scheduler is trusted with pairing,** as it's already trusted with all scheduling ([0088](0088-user-mode-scheduler.md)).
- **Trust stays explicit and mutual,** as 0098 requires. Both programs must have been given the group.
- **0098's question is answered.**
