# 0264. Checkpoints set performance targets, and a missed target becomes a record

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The design pays for IPC on purpose: a user-mode scheduler ([0088](0088-user-mode-scheduler.md)), time read through a service ([0112](0112-time-service.md)), one kernel lock ([0095](0095-one-kernel-lock.md)), and one receiving thread for each endpoint ([0121](0121-callers-help-busy-services.md)). Day 2 estimated an IPC round trip at about 530 cycles ([02-verification](../design/02-verification.md)). Benchmarks are tracked over time but not gated ([0235](0235-kinds-of-tests.md)), and they run on the laptop ([0240](0240-timing-on-laptop.md)). A frozen decision changes through a record that cites a measurement, among other things ([0201](0201-changes-after-freeze.md)). The audit found from 0235's text that with no targets, a slow path never becomes such a measurement ([0256](0256-audit-findings-as-evidence.md)).

## Options

1. **Targets set at checkpoints,** from what they measure.
2. **Fixed targets now,** such as an IPC round trip of at most 1,000 cycles.
3. **Benchmarks as they are.**

## Decision

Option 1.

- From checkpoint 3 on, each checkpoint sets targets for the paths it can measure, from what it measures on the laptop: the IPC round trip and a page fault at checkpoint 3, a time read and lent memory at checkpoint 4 ([0265](0265-lent-memory-library.md)), and the same paths across cores at checkpoint 5.
- Each target names the decisions whose cost it measures, such as 0088, 0095, 0112 and 0121.
- When a later checkpoint misses a target, a record under 0201 cites the measurement. It reopens the decisions the target names, or raises the target and gives the reason.
- Targets and results are kept in 08-roadmap. CI stays ungated, since its runners can't time reliably ([0240](0240-timing-on-laptop.md)).
- This amends 0235, whose benchmarks gain targets, and 0249, whose checkpoints set and check them.

## Consequences

- **A design that turns out too slow produces the evidence that changes it.**
- **Targets come from the system as built,** not from guesses made before any code exists.
- **The checkpoints after steps 9, 14 and 16 are affected** ([08-roadmap](../design/08-roadmap.md)).
