# 0049. When memory runs short, it's shared fairly within each memory priority

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Denial of service is in scope ([0016](0016-threat-model-scope.md)), and there are no quotas ([0023](0023-no-quotas.md)). [04-execution](../design/04-execution.md) asked what happens when one program uses up memory that others need. Under [0025](0025-user-mode-memory.md) and [0026](0026-programs-pay-kernel-objects.md), that becomes how the memory service hands out memory when demand exceeds what's available. CPU time is already shared fairly within each priority ([0033](0033-cpu-fairness.md)).

## Options

1. **Fair share when short.** Any program may use all free memory, and shares count only when memory runs short.
2. **First come, first served.** Requests fail once memory is gone, and nothing is taken back.
3. **Stop the biggest,** like Linux's OOM killer.

For a request from a program already over its fair share while memory is short:

- **It fails.**
- **It's served** by taking memory from programs further over their share.

For priorities:

- **Strict,** as for CPU time in 0033.
- **As weights** that set the size of each program's share.

## Decision

Option 1, with requests from programs over their share failing, and strict priorities. Memory is shared the way CPU time is under 0033.

- While memory is free, any program may use all of it.
- When memory runs short, programs over their fair share within their memory priority ([0062](0062-memory-priority-capability.md)) are asked to give memory back. If they don't, their memory is paged out or they are stopped.
- When real memory is short but reservations still fit in swap, pagers page out memory from programs over their fair share first.
- A request from a program under its fair share is met by taking memory from programs over theirs.
- A request from a program already over its fair share fails while memory is short, with the error from [0048](0048-reserve-on-grant.md).
- Priorities are strict. A program at a higher memory priority is served by taking memory from lower priorities, whatever their shares. Fair shares apply only among programs at the same priority.
- Memory counts against its payer ([0064](0064-memory-belongs-to-payer.md)).
- Nothing applies while memory is free, so this is not a cap ([0012](0012-no-memory-cap.md), [0023](0023-no-quotas.md)).

## Consequences

- **04's memory exhaustion question is answered.**
- **There are two kinds of shortage.** Memory is reserved when granted ([0048](0048-reserve-on-grant.md)), so:
  - When real memory is short but reservations still fit in swap, memory is paged out and nothing has to be stopped.
  - When reservations themselves don't fit, programs over their share are asked to give memory back, and stopped if they don't.
  - Until swap exists, only the second kind can happen.
- **Granting a high memory priority is a trust decision.** It lets a program cause lower-priority programs to be stopped.
- **Taking memory back uses revoke** ([0053](0053-revoke-destroys-derived.md)).
- **Programs need a way to be asked** to give memory back, for example by dropping caches. How they're asked is open ([03-memory](../design/03-memory.md)).
- **What counts as one share is open.** It could be per program, or along the spawn tree so a program can't enlarge its share by spawning children. That's decided together with CPU fairness ([04-execution](../design/04-execution.md)).
- **Stopping a program is the last step.** Its memory goes back to the memory service. Who is told, and whether the program is restarted, belong to fault isolation ([04-execution](../design/04-execution.md)).
