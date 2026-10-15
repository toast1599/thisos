# 0057. A pager's own faults go to its pager, up a chain that ends at the root task

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

A pager must not fault into itself ([0027](0027-user-mode-pagers.md)).

## Options

1. **Faults go up a chain.** A pager's own faults go to its own pager, one level up.
2. **Pagers never fault.** Every pager has all its memory mapped before it runs, and any fault in a pager stops it.

## Decision

Option 1. This is the traditional L4 arrangement.

- A pager's own faults go to its own pager, one level up.
- The chain ends at the root task ([0024](0024-root-task.md)), whose memory is all mapped at boot and which never faults.
- A pager that is set up to handle its own faults hangs only itself and the programs that chose it.

## Consequences

- **Pagers can page their own memory,** like any other program.
- **The root task's memory is mapped by the kernel at boot,** the one exception to [0052](0052-user-mode-page-tables.md).
- **The root task pages the memory service and the default pager** ([0071](0071-three-memory-programs.md)). That keeps the default pager, which depends on the memory service, from paging it.
- **A bad setup harms only those who chose it.** The kernel doesn't look for pagers set up to handle their own faults. Such a setup hangs only the programs that chose that pager ([0056](0056-pager-chosen-at-spawn.md)).
- **03's pager questions are answered.**
