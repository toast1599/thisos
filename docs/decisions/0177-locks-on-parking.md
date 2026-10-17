# 0177. std's locks are built on parking, and parking on notifications

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

std's locks usually rest on a futex, where the kernel checks a lock word in program memory before putting a thread to sleep. The kernel never reads program data ([0072](0072-registers-only.md)). Notifications carry events ([0118](0118-calls-and-notifications.md)). Sleeping and timeouts belong to the scheduler ([0094](0094-one-deadline-per-core.md)).

## Options

1. **Parking on notifications.**
2. **A futex in the kernel,** amending 0072.
3. **A futex service** in user mode, with wait queues kept by address.

## Decision

Option 1.

- Each std thread owns a notification. `thread::park` waits on it, and `unpark` signals it.
- std's mutexes, condition variables and once-cells use std's queue-based versions, which are built on parking, with no futex.
- Sleeping and timed waits are calls to the scheduler, which wakes the thread or cancels its wait ([0094](0094-one-deadline-per-core.md), [0134](0134-scheduler-interface.md)).
- Thread-locals use the thread-pointer register, FS on x86_64 and TPIDR_EL0 on AArch64, which the kernel saves and restores for each thread.

## Consequences

- **0072 holds.** The kernel never reads a lock word.
- **An uncontended lock needs no system call.** A contended one costs a wait and a signal.
- **Each thread pays for one more kernel object,** its notification ([0026](0026-programs-pay-kernel-objects.md)).
- **`thread::spawn` makes the thread and its notification,** then hands the thread to the scheduler ([0134](0134-scheduler-interface.md)).
- **On x86_64, the kernel lets programs set FS themselves,** by turning on the instructions for it.
