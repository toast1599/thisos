# 0034. Only programs that trust each other share a CPU core

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Two hardware threads on one core share branch predictors and caches. This is called SMT or hyper-threading, and it's the hardest place to stop speculative leaks between programs, which are in scope ([0016](0016-threat-model-scope.md)).

## Options

1. **Pair only trusted programs:** run programs on sibling hardware threads only if they trust each other.
2. **Turn SMT off** on every core.
3. **Allow any pairing,** and document the leaks as a limitation.

## Decision

Option 1.

- The scheduler runs programs at the same time on sibling hardware threads only if they trust each other.
- Where that can't be arranged, SMT is turned off.

## Consequences

- **The scheduler needs to know about trust.** It has to know which programs trust each other. How that's expressed is open ([04-execution](../design/04-execution.md)).
- **Some throughput is lost** when no trusting pair is ready to run.
- **SMT handling is architecture-specific** ([0008](0008-architecture-neutral-design.md)). Many AArch64 cores have no SMT at all.
