# 0023. No quotas: a program's resources are limited only by the hardware and what's available

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0012](0012-no-memory-cap.md) ruled out fixed limits built into the design. Denial of service is in scope ([0016](0016-threat-model-scope.md)), and the usual defence is a per-program quota set by an owner or administrator.

## Options

1. Allow quotas set by policy, with none by default.
2. No quotas at all.

## Decision

Option 2.

- No quota limits a program's memory or CPU use, whether built into the design or set by policy.
- Programs are limited only by the hardware and by what's available.

## Consequences

- **Quotas can't defend against denial of service** ([0016](0016-threat-model-scope.md)). What happens when one program uses up memory that others need is open ([04-execution](../design/04-execution.md)).
- **The scheduler still shares CPU time.** How it keeps one program from starving the others is open ([04-execution](../design/04-execution.md)).
- **Resource accounting can still measure use** without capping it, for visibility ([04-execution](../design/04-execution.md)).
