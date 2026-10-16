# 0101. Floating-point and vector registers are saved and restored on every switch

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

Restoring floating-point and vector registers lazily, on a thread's first use after a switch, leaked their contents speculatively on Intel CPUs (LazyFP, 2018). Speculative leaks between programs are in scope ([0016](0016-threat-model-scope.md)).

## Options

1. **On every switch.**
2. **Lazily, on first use.**

## Decision

Option 1. The kernel saves and restores floating-point and vector registers whenever it switches threads, and never restores them lazily.

## Consequences

- **No LazyFP-style leak** ([0016](0016-threat-model-scope.md)).
- **Each switch costs a few hundred cycles more** when vector state is large.
- **Thread objects hold this state,** up to a few KiB, which the thread's program pays for ([0026](0026-programs-pay-kernel-objects.md), [0060](0060-kernel-stack-per-core.md)).
- **Which registers exist, and how they're saved,** is architecture-specific ([0008](0008-architecture-neutral-design.md)).
