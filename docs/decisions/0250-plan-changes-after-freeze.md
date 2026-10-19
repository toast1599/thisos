# 0250. After the freeze, steps change only through a record that cites what building found

- **Status:** Accepted, amended by [0256](0256-audit-findings-as-evidence.md)
- **Date:** 2026-10-19

## Context

The design and the roadmap freeze together ([0255](0255-freeze.md)). After the freeze, a frozen decision changes only through a record that cites what building found, and names the roadmap step it affects ([0201](0201-changes-after-freeze.md)). Day 8 asks what happens when building turns up something the plan missed ([08-roadmap](../design/08-roadmap.md)). Open topics are designed at their steps ([0226](0226-open-topics-at-their-step.md)).

## Options

1. **Records for changes to steps,** with estimates updated at checkpoints.
2. **A living plan,** changed at checkpoints with a change log.
3. **Records for everything,** estimates included.

## Decision

Option 1.

- After the freeze, adding, removing, splitting, merging or reordering steps, or moving a decision to another step, takes a record. The record cites what building found and names the steps affected, as [0201](0201-changes-after-freeze.md) has for the design.
- Estimates, progress and the dates of checkpoints are updated at checkpoints, without records ([0249](0249-estimates-and-checkpoints.md)).

## Consequences

- **The plan holds as firmly as the design,** and changes for the same kind of reason.
- **Estimates follow the work** without a record for each change.
- **The list of open topics changes as the steps do** ([0226](0226-open-topics-at-their-step.md)).
- **Day 8's question on changing the plan is answered,** together with [0201](0201-changes-after-freeze.md).
