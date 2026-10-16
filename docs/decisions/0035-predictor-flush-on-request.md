# 0035. Branch predictors are flushed only on request

- **Status:** Superseded by [0080](0080-predictor-flush-both-ways.md)
- **Date:** 2026-10-15

## Context

Flushing the branch predictors when switching from one program to another stops one program from steering another's speculation. On the development machine, one flush costs 545 cycles. Flushing on every switch would roughly triple the cost of an IPC round trip ([02-verification](../design/02-verification.md)).

## Options

1. **Only on request**, as in Linux's default.
2. **Always**, on every switch between programs.
3. **Never**, relying on the hardware alone.

## Decision

Option 1.

- The kernel flushes the branch predictors when it switches away from a program that has asked for this.
- Otherwise it doesn't flush.

## Consequences

- **Programs holding secrets ask for flushes.** Key-management services are an example. How a program asks is designed with the system calls ([05-interfaces](../design/05-interfaces.md)).
- **IPC between other programs keeps its full speed.**
- **Programs that don't ask accept a risk.** They accept that predictor state may carry over between them and the next program, within the best-effort defence of [0016](0016-threat-model-scope.md).
