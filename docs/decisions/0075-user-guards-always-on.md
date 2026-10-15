# 0075. The kernel's guards against touching or running program memory are always on

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

The kernel never reads program data ([0072](0072-registers-only.md)). The hardware can stop the kernel from accessing or running program memory. On x86_64 that's SMAP and SMEP. On AArch64 it's PAN, plus marking program pages execute-never for the kernel. Most kernels switch the access guard off around copies from programs.

## Options

1. **Always on.**
2. **Switched as needed.**

## Decision

Option 1.

- These guards are on whenever the kernel runs, on every CPU that has them.
- Nothing in the kernel turns them off.

## Consequences

- **A kernel bug that follows a program's pointer faults,** instead of reading, writing or running program memory.
- **This works only because of [0072](0072-registers-only.md).** A later design that had the kernel touch program memory would need a new record.
- **The kernel's own writes aren't blocked.** Clearing memory ([0051](0051-kernel-clears-memory.md)) and setting up the root task at boot ([0052](0052-user-mode-page-tables.md)) go through the kernel's own mappings, which the guards don't block.
- **On AArch64, the kernel marks every program page execute-never for itself** as it checks each mapping ([0052](0052-user-mode-page-tables.md)).
- **CPUs without these guards** run without this protection. Which ones lack them is architecture-specific ([0008](0008-architecture-neutral-design.md)).
