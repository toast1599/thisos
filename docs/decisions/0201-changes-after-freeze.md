# 0201. After the freeze, a decision changes only through a new record that cites what building found

- **Status:** Accepted, amended by [0256](0256-audit-findings-as-evidence.md)
- **Date:** 2026-10-18

## Context

The design and the roadmap freeze together at the end of Day 8 ([08-roadmap](../design/08-roadmap.md)). Until then, an Accepted decision changes through a new record that supersedes it or amends some of its lines, and records are never edited after the fact ([decision log](README.md)). Building will turn up flaws the design missed. Each interface can change freely until its version 1 is released ([0148](0148-freeze-covers-rules.md)). Day 8 asks what happens when building turns up something the plan or the design missed ([08-roadmap](../design/08-roadmap.md)).

## Options

1. **New records with evidence.** A frozen decision changes only through a new record, which cites what building found.
2. **An errata list,** with records amended in batches at the roadmap's checkpoints.
3. **Planned revisions only.** Flaws are collected and fixed together in a planned revision of the design.

## Decision

Option 1.

- After the freeze, a frozen decision changes only through a new record, which supersedes or amends it as before ([0082](0082-corrections.md)).
- The record must cite what building found: a failing test, a measurement or a bug. It also names the roadmap step it affects.
- Like every record, it binds only once the architect accepts it.
- The decision log lists records written after the freeze in a section of their own.

## Consequences

- **The freeze holds without blocking fixes.** A flaw found while building is fixed when it's found, and the reason is on record.
- **Opinion alone doesn't reopen the design.** A change needs evidence from building.
- **Interfaces before their release aren't affected.** They change freely within the frozen design, as 0148 allows.
- **Topics the freeze leaves to later work,** such as rollback protection ([0209](0209-no-rollback-protection.md)), get ordinary records when their time comes.
- **Day 8's question on changing the plan builds on this** ([08-roadmap](../design/08-roadmap.md)).
- **07's question on changes after the freeze is answered.**
