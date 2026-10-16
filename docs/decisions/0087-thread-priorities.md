# 0087. A thread runs at any CPU priority up to its program's

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

[0033](0033-cpu-fairness.md) grants a CPU priority to a program. Programs often want some of their threads to run ahead of others, for example a UI thread ahead of background work.

## Options

1. **Up to the program's priority,** as with seL4's maximum controlled priority.
2. **One priority per program.** Every thread runs at its program's priority.

## Decision

Option 1. Each thread runs at a CPU priority up to the one its program was granted.

## Consequences

- **A program can favour some of its threads over others.**
- **A program still can't raise its own priority** ([0033](0033-cpu-fairness.md)). No thread runs above the priority its program was granted, however many threads it makes.
- **Memory priority stays per program** ([0062](0062-memory-priority-capability.md), [0086](0086-priorities-per-program-shares-per-account.md)).
- **How a thread's priority is set** comes with the interfaces ([05-interfaces](../design/05-interfaces.md)).
