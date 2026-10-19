# 0113. Only programs holding a capability may read the CPU's time counter

- **Status:** Accepted, amended by [0268](0268-counter-by-default.md)
- **Date:** 2026-10-16

## Context

Programs read the time from a time service ([0112](0112-time-service.md)). The scheduler needs the time for every choice it makes ([0094](0094-one-deadline-per-core.md)) and can't call the time service for it, and the time service has to read the hardware itself. Precise timers are what speculative-execution attacks rely on to measure their results, and such leaks are in scope ([0016](0016-threat-model-scope.md)).

## Options

1. **Only with a capability.**
2. **Readable by all.**

## Decision

Option 1.

- A program can read the CPU's time counter only if it holds a capability for it.
- The scheduler and the time service hold it.
- The kernel turns user-mode access to the counter on and off as it switches between programs.

## Consequences

- **Precise timers stay scarce.** Other programs get the time from the time service ([0112](0112-time-service.md)).
- **Switching costs a little more** for programs whose access differs. Turning access on and off is architecture-specific ([0008](0008-architecture-neutral-design.md)).
- **Granting it is a trust decision,** like other capabilities. Profilers are an example of programs that might get it.
- **04's question on the CPU's counter is answered.**
