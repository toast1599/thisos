# 0217. Object-window flushes are batched, at most at a set rate

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

Memory stays mapped in the object window after the object made from it is destroyed. When it's made into pages for a program, the kernel removes its window mapping and flushes every core first ([0096](0096-object-window-lazy-unmap.md)). So a program can loop, making an object, destroying it and mapping the memory as a page, and interrupt every core each time. Denial of service between programs is in scope ([0016](0016-threat-model-scope.md)).

## Options

1. **Batched, at most at a set rate.**
2. **Only the cores that touched it.** The kernel records which cores touched each object.
3. **Accept it,** within 0016's best-effort defence.

## Decision

Option 1.

- The kernel never flushes other cores on demand for the object window.
- It collects window removals, and flushes every core in batches, at most at a set rate.
- Memory waits for the batch that covers it before it can become a program's pages. A request that comes sooner fails with an error, and the caller retries.
- This amends 0096, whose flush before memory becomes pages now happens in these batches.

## Consequences

- **No program can make the kernel interrupt other cores at will** through the object window.
- **Program data still never sits in the kernel's mappings** ([0055](0055-kernel-maps-only-objects.md)).
- **Reusing object memory as pages can wait,** up to one batch. Pagers and the loader retry ([0138](0138-pager-keeps-reservations.md)).
- **A fault on granted memory still never fails for lack of memory** ([0048](0048-reserve-on-grant.md)). A retry isn't a failure.
- **The rate is chosen during implementation.**
