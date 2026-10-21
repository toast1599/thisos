# 0290. If the measured pace shows the plan far off, scope moves to later work by record

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

The roadmap ends at the whole design ([0222](0222-roadmap-to-first-release.md)), and promises no dates; each milestone records the pace it measured ([0277](0277-sized-work-no-dates.md)). Later work stays unscheduled, and the first release states the gaps it leaves ([0251](0251-later-work.md)).

## Options

1. **Scope moves, by record.**
2. **Nothing gives.** The whole design stays the target.
3. **No rule.** The architect decides at each milestone.

## Decision

Option 1.

- At a milestone, a record may move parts of the plan to later work, citing the measured pace.
- In-scope security defences can't move ([0016](0016-threat-model-scope.md), [0202](0202-offline-attackers.md), [0257](0257-outside-attackers.md)), and neither can anything a remaining part needs.
- The first release still states every gap.
- This amends 0222, whose whole design may shrink by such records, and 0251, whose later work gains them.

## Consequences

- **The plan can bend to what building measures,** in the open.
- **Security isn't what gives.**
