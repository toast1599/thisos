# 0278. Each step opens with a design pass, and its code lands as small work items

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

Open topics are designed at the step that needs them ([0226](0226-open-topics-at-their-step.md)). Step 1 needed 11 decisions on topics no record or open topic named ([0272](0272-roadmap-reopened.md)). A step is done when its tests pass, CI passes, its review is finished and its records have their tests ([0228](0228-step-start-and-done.md)). A step can hold hundreds of review hours: step 2 was sized at 315 ([08-roadmap](../design/08-roadmap.md)).

## Options

1. **Designed steps, small work items.**
2. **Steps as they are,** with their open topics designed at their start.
3. **A plan for each day,** like Week 1's design days.

## Decision

Option 1.

- A step opens with a design pass. It settles the step's open topics, and the questions its building may raise, in records before the step's code.
- The step's code then lands as work items. Each is one change, small enough to review in one sitting: up to a few hundred lines of tier 1, about 500 of tier 2, or about 1,000 of tier 3 ([0242](0242-review-tiers.md)).
- A work item is done when CI passes on it, its review is recorded ([0280](0280-reviews-in-progress-log.md)), and its tests name the records they check ([0234](0234-tests-cite-records.md)).
- A step is done when its work items are, and 0228's conditions hold.
- This amends 0226, whose design pass also settles what building may raise, and 0228, whose steps are done through their work items.

## Consequences

- **A step's hidden decisions surface before its code,** as step 1's did only while it was built.
- **Review proceeds in pieces of a known size,** each of them done on its own.
