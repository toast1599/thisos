# 0161. The root task runs alone until it starts the scheduler

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

A thread runs only once a scheduler thread has taken it on ([0134](0134-scheduler-interface.md)). The scheduler is a program that the root task starts ([0088](0088-user-mode-scheduler.md), [0160](0160-root-task-starts-four.md)). The scheduling-control capability is a kernel capability, which the root task holds at boot and passes to the scheduler ([0134](0134-scheduler-interface.md)). Something has to run before the scheduler exists.

## Options

1. **The root task alone,** with the scheduler started first.
2. **A fallback policy in the kernel,** which runs threads in turn within each priority until a scheduler exists.

## Decision

Option 1.

- The kernel starts the root task's first thread on the boot core, and runs it without a scheduler. It's the one thread that runs before a scheduler takes it on. This amends 0134, in the way 0052 makes the root task's address space an exception.
- The root task starts the scheduler before any other program ([0160](0160-root-task-starts-four.md)).
- Through the scheduling-control capability, it names the scheduler's first thread as the boot core's scheduler thread ([0094](0094-one-deadline-per-core.md)). It then moves that capability to the scheduler ([0123](0123-capabilities-in-messages.md)), and gives it a copy of the counter capability, which the time service also needs ([0113](0113-counter-capability.md)).
- It hands its own thread to the scheduler, as any program does ([0134](0134-scheduler-interface.md)).

## Consequences

- **The kernel gains no scheduling policy** ([0019](0019-kernel-minimality.md), [0088](0088-user-mode-scheduler.md)).
- **Only the scheduler holds scheduling control** once it runs, as 0134 requires.
- **Nothing else runs until the scheduler does.** Starting the scheduler is the only work done without one.
- **Other cores start later** ([0165](0165-core-objects.md)), each with its scheduler thread named first.
- **0088's and 0134's questions on how the scheduler is started are answered.**
