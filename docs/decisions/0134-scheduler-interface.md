# 0134. Threads run once the scheduler takes them on, and the kernel reports to it in registers

- **Status:** Accepted, amended by [0161](0161-root-task-before-scheduler.md)
- **Date:** 2026-10-17

## Context

The kernel picks the priority, and the user-mode scheduler picks the thread within it ([0090](0090-priorities-in-kernel.md)), with a scheduler thread on each core ([0093](0093-scheduler-per-core.md)). Several parts were left to the interfaces:

- how a thread's priority is set ([0087](0087-thread-priorities.md));
- how the kernel's run-time reports reach the scheduler within registers, including chains of calls ([0092](0092-scheduler-keeps-time.md));
- how the scheduler cancels a wait that timed out ([0094](0094-one-deadline-per-core.md)).

The scheduler also has to learn which threads exist.

## Options

1. **Enrol, tag, report.** A program hands each new thread to the scheduler, which binds it with a tag of its own, and the kernel reports by tag.
2. **The kernel enrols threads.** The kernel puts a capability for each new thread into the scheduler's capability space.

## Decision

Option 1.

- A thread runs only once a scheduler thread has taken it on. The program hands the scheduler the thread's capability, as std's `thread::spawn` does.
- The scheduler binds the thread to a core, with a tag of its own choosing, through a scheduling-control capability that only it holds.
- The kernel keeps a running total of CPU time for each thread, counted against the thread whose time was used ([0085](0085-clients-pay-cpu-time.md), [0121](0121-callers-help-busy-services.md)).
- Every entry to a core's scheduler thread carries, in registers, why it was entered, the tags of the threads involved and their running totals.
- The scheduler cancels a thread's wait through the thread's capability and the scheduling-control capability ([0094](0094-one-deadline-per-core.md)).
- A thread's priority is set through its capability, together with a priority capability at least as high ([0087](0087-thread-priorities.md), [0090](0090-priorities-in-kernel.md)).

## Consequences

- **Reports fit in registers, whatever happened between them.** Calls, replies and lending change only whose time is used, and the running totals already count that ([0092](0092-scheduler-keeps-time.md)).
- **The scheduler names threads by its own tags,** which the kernel never interprets.
- **A thread that nobody hands to the scheduler never runs.** The loader hands over a new program's first thread ([0031](0031-user-mode-loader.md)).
- **No thread runs above its program's priority** ([0087](0087-thread-priorities.md)). The kernel checks the priority capability.
- **The kernel still has no account object** ([0092](0092-scheduler-keeps-time.md)). A running total is one field in each thread object.
- **The scheduling-control capability is a kernel capability.** The root task holds it at boot ([0024](0024-root-task.md)) and passes it to the scheduler. How is designed with the boot path ([06-system](../design/06-system.md)).
- **0087's, 0092's and 0094's questions are answered.**
