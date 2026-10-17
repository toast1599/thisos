# 0091. Calls and replies switch directly between client and service

- **Status:** Accepted, amended by [0219](0219-wakeup-preemption.md)
- **Date:** 2026-10-16

## Context

Under [0085](0085-clients-pay-cpu-time.md), a call carries the client's time and priority into the service. [0088](0088-user-mode-scheduler.md) sends scheduling choices to a user-mode scheduler, and left open whether calls and replies go through it too.

## Options

1. **Calls switch directly.** Only preemption, blocking and other wakeups go to the scheduler.
2. **Every switch asks the scheduler.**

## Decision

Option 1.

- A call hands the core straight to the service, on the client's time, and the reply hands it back.
- Only preemption, blocking and other wakeups go to the scheduler.

## Consequences

- **IPC keeps its cost.** Neither a call nor a reply adds a trip to the scheduler.
- **The kernel reports time spent this way to the scheduler** ([0092](0092-scheduler-keeps-time.md)).
- **The mechanism comes with the first IPC design** ([0021](0021-ipc-simple-upgradable.md), [05-interfaces](../design/05-interfaces.md)).
