# 0090. Strict priorities stay in the kernel, and the scheduler orders threads within each priority

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

[0088](0088-user-mode-scheduler.md) moved scheduling policy to a user-mode scheduler, and left open whether strict priorities ([0033](0033-cpu-fairness.md)) stay in the kernel. If the scheduler handled them, every wakeup of a higher-priority thread, such as a driver woken by an interrupt, would wait for a trip to it.

## Options

1. **The kernel keeps them.** The scheduler orders threads within each priority.
2. **The scheduler handles them.** Priority capabilities come from the scheduler.

## Decision

Option 1.

- The kernel always runs a ready thread at the highest priority that has one.
- Within each priority, the user-mode scheduler picks which thread runs next.
- This amends 0088: the kernel picks the priority, and the scheduler picks the thread within it.

## Consequences

- **A woken thread at a higher priority runs at once,** without a trip to the scheduler.
- **0033 stays as written.** Priority capabilities are kernel capabilities, and the root task holds them all at boot ([0024](0024-root-task.md)).
- **The kernel applies 0085's priority rule.** A service working on a client's call runs at the client's priority ([0085](0085-clients-pay-cpu-time.md)).
- **The scheduler is asked only to choose among threads at the same priority.**
