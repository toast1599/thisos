# 0056. A program's pager is chosen when it's spawned

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Every program needs a pager ([0027](0027-user-mode-pagers.md)). Under [0022](0022-explicit-spawn.md), a program gets exactly what its creator hands it.

## Options

1. **Chosen at spawn.** Whoever spawns a program names its pager.
2. **One system pager** for every program.

## Decision

Option 1.

- Whoever spawns a program names its pager. Normally that's the default pager ([0071](0071-three-memory-programs.md)).
- A program can also be given a different pager, or have a pager thread of its own, as a language runtime that manages its own heap might.

## Consequences

- **The pager is part of a program's startup information** ([04-execution](../design/04-execution.md)).
- **Runtimes and databases can set their own paging policy.**
- **A pager thread can't page itself.** Its own faults go to its own pager ([0057](0057-pager-fault-chain.md)).
- **Naming a pager is a trust decision.** A pager sees a program's faults and decides what memory it gets, so the spawner chooses whom the new program relies on.
- **Whether a pager can be changed after spawn** is part of the capability design ([05-interfaces](../design/05-interfaces.md)).
