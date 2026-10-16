# 0114. Each core's scheduler thread runs above every priority it schedules

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

When a core's timer deadline passes, the kernel switches to that core's scheduler thread ([0094](0094-one-deadline-per-core.md)). The kernel also always runs a ready thread at the highest priority that has one ([0090](0090-priorities-in-kernel.md)). Both hold only if the scheduler outranks the threads it schedules.

## Options

1. **Above all.** Each core's scheduler thread runs above every priority it schedules.
2. **Up to its own priority.** The deadline wakes the scheduler at its own priority, and threads above it are never time-sliced.

## Decision

Option 1. Each core's scheduler thread runs above every priority it schedules.

## Consequences

- **0090 and 0094 agree.** Switching to the scheduler when the deadline passes is running the highest-priority ready thread.
- **Every priority can be time-sliced.**
- **How the scheduler gets that priority** comes with the boot path ([06-system](../design/06-system.md)).
