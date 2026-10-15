# 0033. CPU fairness: fair sharing within each priority, and priorities as capabilities

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Denial of service is in scope ([0016](0016-threat-model-scope.md)), and there are no CPU budgets ([0023](0023-no-quotas.md)). Something still has to stop one program from starving the others.

## Options

These can be combined:

1. **Fair sharing within each priority.**
2. **Priorities as capabilities:** a program runs at a priority it was granted and can never raise it itself.
3. **A watchdog** that acts on programs that hog the CPU.

## Decision

Options 1 and 2.

- Programs at the same priority share CPU time evenly.
- A program runs at a priority it was granted through a capability, and can never raise that priority itself.
- There is no watchdog.

## Consequences

- **Only higher-priority programs can starve others.** A program can starve only programs below it, and only if it was granted a higher priority.
- **Holding the capabilities means granting priorities.** Whoever holds the priority capabilities decides who gets high priorities. The root task starts out with all of them ([0024](0024-root-task.md)).
- **No cap on CPU use.** A program can still use all the CPU when nothing else wants it, so [0023](0023-no-quotas.md) holds.
- **Details are open** ([04-execution](../design/04-execution.md)):
  - time slices, preemption and SMP;
  - priority inversion, when a high-priority program waits on a lower-priority service.
