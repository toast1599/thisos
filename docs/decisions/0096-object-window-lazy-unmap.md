# 0096. The object window keeps memory mapped until it becomes a program's pages

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

Under [0055](0055-kernel-maps-only-objects.md) and [0067](0067-object-window-offset.md), a kernel object is mapped in the object window only while it's a kernel object. Destroying one unmaps it, which needs a TLB flush on every core, so a program that makes and destroys objects in a loop can interrupt every core each time.

The window maps memory at its physical address plus a fixed offset, so a mapping left behind always points to the right memory. It matters only once the memory holds program data, which 0055 keeps out of the kernel's mappings.

## Options

1. **Flush before reuse as pages.**
2. **Flush on every destruction,** as 0055 and 0067 say.

## Decision

Option 1.

- Memory stays mapped in the object window after the object made from it is destroyed.
- When the memory is made into pages for a program, the kernel removes its window mapping and flushes every core before the program can use the pages.
- Several such removals can share one flush.
- This amends 0067, whose memory stays mapped until it becomes a program's pages, and 0055, whose destroyed objects are no longer unmapped at once.

## Consequences

- **Making and destroying objects doesn't interrupt other cores.** Only turning memory that held objects into pages does.
- **Program data still never sits in the kernel's mappings** ([0055](0055-kernel-maps-only-objects.md)).
- **The kernel still clears memory before the program gets it** ([0051](0051-kernel-clears-memory.md)).
- **Memory remade into a kernel object needs no new mapping** if it's still mapped.
