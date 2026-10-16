# 0148. The freeze covers the interface design, and each interface's version 1 is fixed when it's released

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

[0007](0007-abi-calls-and-interfaces.md) proposed that the freeze at the end of Week 1 cover the ABI's rules and version 1 of the core interfaces. The freeze now comes after Day 8, and covers the design and the roadmap together ([08-roadmap](../design/08-roadmap.md)). Nothing will have been built by then to test any interface's version 1 against.

## Options

1. **The rules now, version 1 at release.**
2. **Version 1 now,** written out in full this week.

## Decision

Option 1.

- The freeze covers the interface design: the system library as the boundary ([0117](0117-system-library.md)), the message shape ([0119](0119-message-eight-words.md)), the capability model ([0126](0126-capability-space-radix-table.md) to [0131](0131-namespace-path-prefixes.md)), the IDL ([0142](0142-own-idl.md)) and the versioning rules ([0143](0143-add-only-versioning.md)).
- Each interface's version 1 is fixed when it's first released, after it's been built and tested. From then on it only grows ([0143](0143-add-only-versioning.md)).

## Consequences

- **Interfaces are tested before they're fixed.** Mistakes found while building them cost nothing to correct.
- **Until its release, an interface can change freely** within the frozen design.
- **The roadmap says when each interface is released** ([08-roadmap](../design/08-roadmap.md)).
- **0007's freeze of version 1 is replaced** ([0144](0144-c-calls-and-what-crosses.md)).
