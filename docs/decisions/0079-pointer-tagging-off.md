# 0079. Hardware pointer tagging stays off for now

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

AArch64's top-byte ignore, Intel's linear address masking and AMD's upper address ignore let programs keep tags in pointers without clearing them before use. Research in 2023 (SLAM) found that these features can widen speculative leaks, which are in scope ([0016](0016-threat-model-scope.md)).

## Options

1. **Off for now.**
2. **On where available.**

## Decision

Option 1.

- Hardware pointer tagging stays off on both architectures.
- Programs that tag pointers clear the tags before using them.

## Consequences

- **These features can't widen speculative leaks** ([0016](0016-threat-model-scope.md)).
- **Turning tagging on later won't break programs** that don't use it. Doing so needs a new record.
- **Runtimes that tag pointers pay a little** to clear the tags, as they do on most systems today.
- **Turning tagging on is architecture-specific** ([0008](0008-architecture-neutral-design.md)).
