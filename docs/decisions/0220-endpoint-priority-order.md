# 0220. Callers waiting on an endpoint are served by priority, then arrival

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

A caller whose service isn't ready waits in its own thread object ([0118](0118-calls-and-notifications.md)). A caller that finds its service busy lends it its priority and time, and 0121 promises that a high-priority caller then waits at most for the call already in progress ([0121](0121-callers-help-busy-services.md)). That holds only if waiting callers are served by priority, and no record orders them. In arrival order, a high-priority caller would also wait for every low-priority call queued ahead of it, run on its own time.

## Options

1. **By priority, then arrival,** as in seL4.
2. **Arrival order,** with 0121's promise corrected.

## Decision

Option 1.

- Callers waiting on an endpoint are served highest priority first.
- Within a priority, they're served in the order they arrived.

## Consequences

- **0121's promise holds.** A high-priority caller waits at most for the call in progress.
- **Low-priority callers can wait behind a stream of higher-priority ones,** as strict priorities already allow ([0033](0033-cpu-fairness.md)).
- **Callers at the same priority are served in turn.**
