# 0098. Programs share a core only if both hold the same core-sharing capability

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

Only programs that trust each other run at the same time on sibling hardware threads ([0034](0034-smt-trusted-pairs.md)), and the scheduler, with a thread on each core, does the pairing ([0088](0088-user-mode-scheduler.md), [0093](0093-scheduler-per-core.md)). How programs show that they trust each other was open.

## Options

1. **A trust capability.** Programs share a core only if both hold the same core-sharing capability.
2. **Same account.** All programs in one account ([0084](0084-shares-belong-to-accounts.md)) may share a core.
3. **Same program only.** Only a program's own threads share a core.

## Decision

Option 1.

- Two programs run on sibling hardware threads only if both hold the same core-sharing capability.
- Spawners hand it out, and the scheduler honours it.
- A program without one shares a core only with its own threads.

## Consequences

- **Trust is explicit and mutual.** Neither program can pair itself with the other alone.
- **Handing it out is a trust decision,** made by the spawner like any other capability it passes on ([0022](0022-explicit-spawn.md)).
- **Programs are kept apart by default.** A sandboxed program sits next to nothing it wasn't paired with, even in its own session's account.
- **How the capability is made and checked** comes with the interfaces ([05-interfaces](../design/05-interfaces.md)).
- **04's question on how the scheduler learns which programs trust each other is answered.**
