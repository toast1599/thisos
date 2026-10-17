# 0173. Programs reach the time service through the namespace entry `svc/time`

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Every reading of the time is a call to a user-mode time service ([0112](0112-time-service.md)). Only programs that hold a capability may read the CPU's counter, and the time service is one of them ([0113](0113-counter-capability.md)). Precise timers help speculative-execution attacks, which are in scope ([0016](0016-threat-model-scope.md)).

## Options

1. **A namespace entry.**
2. **A capability in every program's startup data,** which amends 0108.

## Decision

Option 1.

- std reaches the time service through a namespace entry with a fixed name, `svc/time` ([0131](0131-namespace-path-prefixes.md)).
- Spawners pass it on like any entry. A program without it can't read the time, and std's `Instant::now` panics.
- The time service reads the CPU's counter itself for monotonic time ([0113](0113-counter-capability.md)), and learns the counter's rate from the CPU.
- It reads wall-clock time from a clock driver at start, and can correct it later from network time.

## Consequences

- **A sandbox can be given no clock at all.**
- **The kernel stays out of keeping time** ([0112](0112-time-service.md)).
- **The clock driver is an ordinary driver:** on x86_64 the CMOS clock, through I/O ports ([0168](0168-device-memory-and-ports.md)), and on AArch64 whichever clock device the firmware tables name.
- **std's `Command` passes the entry on by default,** with the rest of the namespace ([0179](0179-command-copies-namespace.md)).
- **0112's question is answered.**
