# 0103. A fault with nowhere to go suspends the thread, and in the root task halts the system

- **Status:** Accepted, amended by [0115](0115-root-task-fault-boot-setting.md)
- **Date:** 2026-10-16

## Context

A thread's faults go to its pager ([0081](0081-pager-per-thread.md)) or its exception handler ([0102](0102-exception-handler.md)). Some faults have nowhere to go: the handler may no longer exist, and the root task has no pager ([0057](0057-pager-fault-chain.md)).

## Options

1. **Suspend the thread,** and halt the system for the root task.
2. **Suspend every thread of its program,** and halt the system for the root task.

## Decision

Option 1.

- A thread whose fault has no handler is suspended, so whoever supervises its program can notice and act.
- A fault in the root task halts the system with a log message.

## Consequences

- **A thread never runs on past a fault nobody handled.**
- **The root task is in the trusted base** ([0024](0024-root-task.md)), so the system doesn't go on after it faults.
- **How a supervisor notices a suspended thread** comes with the interfaces ([0104](0104-supervisor.md), [05-interfaces](../design/05-interfaces.md)).
