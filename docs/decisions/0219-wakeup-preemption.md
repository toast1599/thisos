# 0219. A wakeup enters the scheduler only if the woken thread may run before the running one

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The kernel always runs a ready thread at the highest priority that has one, and a woken thread at a higher priority runs at once, without a trip to the scheduler ([0090](0090-priorities-in-kernel.md)). Yet under 0091, only preemption, blocking and other wakeups go to the scheduler, so every wakeup that isn't a call or a reply does ([0091](0091-direct-switch-on-calls.md)). Each core's scheduler thread runs above every priority ([0162](0162-scheduler-above-priorities.md)). So a low-priority program on another core can wake its own threads, bound to a busy core, in a stream, and each wakeup interrupts that core's high-priority thread for a trip to the scheduler.

## Options

1. **Only if it may run first.** A woken thread below the running thread's priority is only marked ready.
2. **Every wakeup, charged.** Every wakeup still goes to the scheduler, which charges the trip to the waker's account.

## Decision

Option 1.

- A woken thread above the running thread's priority runs at once ([0090](0090-priorities-in-kernel.md)).
- A woken thread at the running thread's priority goes to the core's scheduler, which decides, and charges the trip to the waker's account ([0092](0092-scheduler-keeps-time.md)).
- A woken thread below the running thread's priority is only marked ready. Its core isn't interrupted, and the scheduler isn't entered. The scheduler learns of it at its next entry.
- This amends 0091: a wakeup goes to the scheduler only when the woken thread has the running thread's priority.

## Consequences

- **No program can preempt a higher-priority thread** by waking threads below it.
- **0090 and 0091 agree.**
- **The scheduler collects such wakeups at its next entry,** with calls that each fit in registers ([0072](0072-registers-only.md), [0134](0134-scheduler-interface.md)).
- **Waking a thread on a busy core costs that core nothing** until the thread can run.
