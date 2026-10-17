# 0185. Swap goes to a partition, through the block-storage service

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Swap is a feature of the default pager. It comes later, and pages out through the storage service ([0077](0077-swap-in-default-pager.md)). Swapping must never wait on memory that's itself been swapped out.

## Options

1. **A swap partition,** through the block-storage service.
2. **A swap file** in the filesystem.
3. **Decide when swap is built.**

## Decision

Option 1.

- Swapped pages go to a swap partition, which the default pager reaches through the block-storage service, beneath the filesystem.
- Programs on the swap path are never swapped out: the programs the root task pages ([0164](0164-root-task-memory-pool.md)), the IOMMU service ([0169](0169-iommu-service.md)), the drivers that storage depends on and the block-storage service.

## Consequences

- **Swapping never waits on swapped-out memory.**
- **The filesystem can be swapped out,** since swapping doesn't go through it.
- **The swap partition's size is fixed** when it's made.
- **Swapped pages are still encrypted,** with a key made at boot ([0077](0077-swap-in-default-pager.md)).
- **0077's question is answered.**
