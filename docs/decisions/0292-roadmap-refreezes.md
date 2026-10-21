# 0292. The redesigned roadmap freezes with the tag roadmap-2

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

The roadmap reopened on what step 1 found ([0272](0272-roadmap-reopened.md)), and records 0273 to 0291 redesign it. After Day 8's freeze, a step changes only through a record that cites what building found ([0250](0250-plan-changes-after-freeze.md)), and the `freeze` tag marks the frozen design ([0255](0255-freeze.md)).

## Options

1. **A new tag, with 0250's rule.**
2. **A living plan,** revised at each milestone.
3. **Frozen until m1,** then reopened once.

## Decision

Option 1.

- The redesigned roadmap freezes with this change's commit, which is tagged `roadmap-2`. The `freeze` tag stays where it is.
- From then on, steps and milestones change only through a record that cites what building found, as 0250 says.
- Milestone results, the measured pace and forecasts are recorded without records ([0277](0277-sized-work-no-dates.md)). Open topics and later work keep their rules ([0226](0226-open-topics-at-their-step.md), [0251](0251-later-work.md)).
- 08-roadmap's status is Frozen again.
- This amends 0250, whose rule covers milestones too, and 0255, whose `freeze` tag now marks the design, beside `roadmap-2` for the roadmap.

## Consequences

- **Building starts from a fixed plan,** at step 1 and the two spikes ([0282](0282-spikes-and-risks.md)).
- **The plan changes as the design does,** on what building finds.
