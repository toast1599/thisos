# 0164. The root task keeps back a pool of memory for the programs it pages

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The root task handles faults in the memory service and the default pager ([0057](0057-pager-fault-chain.md), [0071](0071-three-memory-programs.md)). Under [0160](0160-root-task-starts-four.md), it also starts the scheduler before the memory service exists, so it pages the scheduler too. The default pager gets the memory for the programs it pages from the memory service ([0071](0071-three-memory-programs.md), [0137](0137-memory-capability-per-program.md)). Each program normally has its own capability to the memory service ([0137](0137-memory-capability-per-program.md)).

## Options

1. **A pool kept back at boot** for all of them.
2. **A pool for the scheduler and the memory service only.** The default pager gets its own memory through the memory service, at the highest memory priority.

## Decision

Option 1.

- At boot, the root task keeps back a pool of memory for itself and the programs it pages: the scheduler, the memory service and the default pager.
- The pool's size comes from the boot manifest ([0160](0160-root-task-starts-four.md)).
- Everything else goes to the memory service.
- These four programs' memory is outside the memory service's books, and they have no memory capability of their own. This amends 0137, under which every program has one.

## Consequences

- **They're never sent notices or stopped** to free memory ([0049](0049-memory-fair-share.md), [0076](0076-memory-notice-deadline.md)).
- **None of them depends on another for its own memory.** The default pager still gets its programs' memory from the memory service ([0137](0137-memory-capability-per-program.md)).
- **The pool is sized for them.** Their per-client state is lent by their clients ([0050](0050-clients-pay.md)), so their own needs stay roughly fixed.
- **The root task takes their memory back directly,** as the payer of their memory, if it has to stop one of them ([0064](0064-memory-belongs-to-payer.md), [0105](0105-supervisor-reclaims.md)).
- **The memory service's figures leave them out** ([0111](0111-resource-accounting.md)). The root task can report the pool.
- **0071's question on how the root task backs their memory is answered.**
