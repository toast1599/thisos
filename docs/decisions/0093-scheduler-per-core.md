# 0093. The scheduler has a thread on each core, which decides for that core

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

[0088](0088-user-mode-scheduler.md) left open whether there is one scheduler for each core or one for all cores.

## Options

1. **One per core.**
2. **One for all cores.**

## Decision

Option 1.

- The scheduler program has a thread on each core, which makes the choices for that core.
- Moving threads between cores is the scheduler's policy.

## Consequences

- **A core's choices never wait on another core.**
- **The scheduler's threads coordinate in user mode** where cores interact, such as moving threads and pairing programs on sibling hardware threads ([0034](0034-smt-trusted-pairs.md)).
- **0088's question on one scheduler or several is answered.**
