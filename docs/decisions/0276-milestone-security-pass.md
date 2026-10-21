# 0276. Each milestone passes a written security pass before it's tagged

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

Speculative leaks and denial of service between programs are in scope, and so are attackers who get the machine while it's off and attackers who reach it from outside ([0016](0016-threat-model-scope.md), [0202](0202-offline-attackers.md), [0257](0257-outside-attackers.md)). Each defence has an attack test ([0235](0235-kinds-of-tests.md)). Day 7's final review walked each in-scope threat against the design, and found seven gaps ([07-engineering](../design/07-engineering.md)). A milestone passes a security pass before it's tagged ([0274](0274-milestone-definition.md)).

## Options

1. **A threat walk, with gaps settled.**
2. **A fixed checklist,** ticked at each milestone.
3. **An outside review** of each milestone.

## Decision

Option 1. Before a milestone's tag, its progress-log entry holds a written security pass that:

- walks each in-scope threat against what the milestone adds: speculation, denial of service, devices with their DMA and interrupts, failures of trusted services, input from outside, and attackers with the machine off;
- confirms that each defence the milestone builds has its attack test;
- turns each gap into a record, which fixes it or accepts it with a reason, before the tag.

## Consequences

- **The threat model is checked against the system as built,** at each milestone, as Day 7 checked it against the design.
- **No gap reaches the next milestone without a record.**
