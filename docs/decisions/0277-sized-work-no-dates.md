# 0277. The roadmap sizes the work and promises no dates

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

Each step is estimated by the code it adds, by review tier, and the review time that needs. From checkpoint 1 on, the pace measured on earlier steps turns that into calendar dates ([0249](0249-estimates-and-checkpoints.md)). One reviewer reviews every change ([0242](0242-review-tiers.md)), and on 2026-10-20 the hours a week that review can get couldn't be given.

## Options

1. **Sized work, no dates.**
2. **Dates from the measured pace,** as 0249 has it.
3. **Target dates now,** moved when they slip.

## Decision

Option 1.

- 08-roadmap sizes each step in lines, by tier, and in review hours, as 0249 does. It sets no dates.
- Each milestone records the review hours it took, and the lines it reviewed by tier ([0274](0274-milestone-definition.md)).
- From the pace measured so far, 08-roadmap gives the remaining work as a range of review hours and of time. The range is a forecast, never a commitment.
- Sizes and forecasts are revised at each milestone, without records ([0250](0250-plan-changes-after-freeze.md)).
- This amends 0249, whose checkpoints set no calendar dates.

## Consequences

- **No date can slip, since none is promised.**
- **The forecast sharpens with each milestone,** from what building measured.
