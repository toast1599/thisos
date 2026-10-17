# 0162. A core's scheduler thread runs above every priority, by a rule in the kernel

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Each core's scheduler thread runs above every priority it schedules ([0114](0114-scheduler-priority.md)). The kernel already knows which thread that is on each core, since it switches to it when the core's deadline passes ([0094](0094-one-deadline-per-core.md)).

## Options

1. **A rule in the kernel.**
2. **A reserved top priority,** which the root task gives the scheduler and never hands to anyone else.

## Decision

Option 1.

- A thread named as a core's scheduler thread, through the scheduling-control capability, runs above every priority while it's ready.
- No priority capability reaches that level.

## Consequences

- **0114 holds whatever priorities are handed out.** Nothing has to be reserved, and no mistake in handing out priorities can break it.
- **The kernel adds one check** to its choice of what runs next: the core's scheduler thread first, then the highest priority with a ready thread ([0090](0090-priorities-in-kernel.md)).
- **0114's question is answered.**
