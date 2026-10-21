# 0281. A step's status is Not started, In progress or Done

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

08-roadmap marks a step done ([0228](0228-step-start-and-done.md)), and its table used Not started, without defining either word ([0272](0272-roadmap-reopened.md)). Each step opens with a design pass, then lands work items ([0278](0278-design-pass-work-items.md)).

## Options

1. **Five stages:** Not started, Designing, Building, In review, Done.
2. **Three:** Not started, In progress, Done.

## Decision

Option 2.

- A step is **Not started** until its design pass writes its first record, or its first work item lands.
- It's **In progress** from then on. The progress log says whether it's designing, building or in review ([0279](0279-progress-log.md)).
- It's **Done** once every condition of 0228 holds, and the log shows the evidence.
- A milestone takes the same three words.
- This amends 0228, whose steps take these words.

## Consequences

- **The tables stay simple,** and the log holds the detail.
