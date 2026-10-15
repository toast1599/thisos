# 0027. Page faults are handled by user-mode pagers

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

A page fault happens when a program touches memory that isn't mapped. Kernels in the L4 family forward faults to a pager program, which decides what to map.

## Options

1. **A user-mode pager** that the kernel forwards faults to.
2. **The kernel** resolves faults itself.

## Decision

Option 1.

- The kernel forwards each page fault to the faulting program's pager, a user-mode program, which decides what to map.
- The kernel only maps what it's told to.

## Consequences

- **Paging features live in the pager, not the kernel.** Demand paging, copy-on-write, memory-mapped files and swapping are all pager features ([03-memory](../design/03-memory.md)).
- **Faults cost IPC.** A fault costs IPC round trips to the pager, so workloads that fault often depend on IPC speed ([0021](0021-ipc-simple-upgradable.md)).
- **Every program needs a pager.** Which pager a program has is part of its startup information ([04-execution](../design/04-execution.md)).
- **A pager must not fault into itself.** How that's arranged is open ([03-memory](../design/03-memory.md)).
- **Stack overflow in user mode goes to the pager.** Hitting a guard page is a fault, so it reaches the pager.
