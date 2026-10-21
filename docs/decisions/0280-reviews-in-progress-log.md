# 0280. Reviews are recorded in the progress log

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

Changes are reviewed in three tiers, and tier 1 against a checklist ([0242](0242-review-tiers.md)). Maintainers push to main after a local `ci` run, so there's no pull-request approval to point to ([0238](0238-pushes-and-pull-requests.md)). Progress is recorded in `docs/progress.md` ([0279](0279-progress-log.md)).

## Options

1. **In the progress log.**
2. **A Reviewed-by trailer on each commit.**
3. **Pull requests for every change.**

## Decision

Option 1.

- The day's section names each work item reviewed: its commits, its tier and its reviewer. For tier 1 it also records 0242's checklist, item by item.
- A change isn't reviewed until an entry says so.
- This amends 0242, whose reviews are recorded there.

## Consequences

- **Every review is on record,** with what it covered.
- **The rule holds for any number of reviewers** ([0242](0242-review-tiers.md)).
