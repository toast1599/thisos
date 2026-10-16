# 0080. Branch predictors are flushed when switching to or away from a program that asks

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

[0035](0035-predictor-flush-on-request.md) decided that the kernel flushes the branch predictors only on request, when it switches away from a program that asked. Its purpose was to stop one program from steering another's speculation.

A consistency review on 2026-10-16 found that a flush on switching away doesn't serve that purpose. It keeps the program's branch history from whatever runs next. But whatever ran before the program can still train the predictors and steer its speculation, which is the main form of Spectre variant 2. Linux's default, which 0035 cites, flushes when either the outgoing or the incoming program asked.

## Options

1. **Both directions:** when switching to a program that asked, and when switching away from it.
2. **Only when switching to it.**
3. **Keep 0035 as written:** only when switching away.

## Decision

Option 1. This supersedes 0035.

- The kernel flushes the branch predictors when it switches to a program that has asked for this, and when it switches away from one.
- Otherwise it doesn't flush, as in 0035.

## Consequences

- **A program that asks can't be steered** by whatever ran before it, and whatever runs after it can't read its branch history.
- **A round trip with it pays two flushes.** On the development machine that's about 1,090 cycles, against about 530 for the round trip itself ([02-verification](../design/02-verification.md)).
- **IPC between programs that don't ask keeps its full speed,** as under 0035.
- **Programs holding secrets ask for flushes,** as under 0035. Key-management services are an example. How a program asks is designed with the system calls ([05-interfaces](../design/05-interfaces.md)).
- **Programs that don't ask accept a risk,** within the best-effort defence of [0016](0016-threat-model-scope.md).
- **Which switches count as switching between programs** depends on what a program is to the kernel ([04-execution](../design/04-execution.md)).
