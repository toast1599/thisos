# 0234. Tests name the records they check, and the build tool maps every decision to its tests

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Day 8 asks how every Accepted decision is mapped to the step that implements it and the tests that check it, so that nothing in the design is left unbuilt or untested ([08-roadmap](../design/08-roadmap.md)). A step is done only when its decisions have their tests ([0228](0228-step-start-and-done.md)).

## Options

1. **Tests cite records, and a tool checks.**
2. **A table kept by hand.**
3. **Steps list their records,** and tests aren't mapped.

## Decision

Option 1.

- Each test names the records it checks, in a form the build tool reads.
- The build tool lists every Accepted record with its step and its tests.
- A step isn't done while a decision it implements has no test ([0228](0228-step-start-and-done.md)).
- A record with nothing to test, such as the license, is marked so in the map, with the reason.
- [08-roadmap](../design/08-roadmap.md) keeps the table that maps each record to its steps.

## Consequences

- **Coverage of the design is checked by a tool,** not by memory.
- **Untested decisions show up before a step is done.**
- **The map grows with every step,** and is complete at the first release ([0222](0222-roadmap-to-first-release.md)).
- **Day 8's question on coverage of the design is answered.**
