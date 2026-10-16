# 0100. The kernel applies each CPU's documented mitigations for speculative-execution flaws

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

[0047](0047-kernel-mapping.md) unmaps the kernel on CPUs affected by Meltdown, but no rule covered the other speculative-execution flaws, such as MDS, L1TF, Retbleed and BHI. Leaks between programs and from programs into the kernel are in scope ([0016](0016-threat-model-scope.md)).

## Options

1. **Each CPU's mitigations.** One rule covers every known flaw.
2. **A record per flaw.**

## Decision

Option 1.

- Architecture-specific code detects which known speculative-execution flaws a CPU has.
- It applies the vendor's documented mitigation for each, on kernel entry and exit and on switches, as 0047 does for Meltdown.
- Newly published flaws are handled the same way, without a record each.

## Consequences

- **0047 is one case of this rule.**
- **The cost differs by CPU.** Detection and mitigations are architecture-specific code ([0008](0008-architecture-neutral-design.md)).
- **New mitigations reach devices with the next system image** ([0042](0042-updates.md)).
