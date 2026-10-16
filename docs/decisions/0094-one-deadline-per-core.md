# 0094. Each core has one timer deadline, set by its scheduler thread

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

A user-mode scheduler with a thread on each core chooses among threads at the same priority ([0088](0088-user-mode-scheduler.md), [0090](0090-priorities-in-kernel.md), [0093](0093-scheduler-per-core.md)), and the kernel owns the timer ([0032](0032-minimal-hardware-discovery.md)). Threads need to sleep, waits need timeouts, and the scheduler needs to preempt a thread when its time is up.

## Options

1. **One deadline per core,** set by that core's scheduler thread.
2. **Deadlines on kernel waits** as well, so the kernel wakes a timed-out thread itself.
3. **Timer objects** that fire and are waited on like other events, as in Zircon.

## Decision

Option 1.

- The kernel keeps one timer deadline for each core, set by that core's scheduler thread.
- When the deadline passes, the kernel switches to that core's scheduler thread.
- Sleeping, timeouts and time slices are all the scheduler's. It wakes sleeping threads and cancels waits that time out.

## Consequences

- **The kernel's timer state is one deadline per core.**
- **Sleeping and timing out cost a trip to the scheduler.**
- **The scheduler needs a way to cancel a thread's wait,** which comes with the interfaces ([05-interfaces](../design/05-interfaces.md)).
- **04's questions on sleeping, timeouts and preemption are answered.**
