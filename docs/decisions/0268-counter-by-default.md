# 0268. Programs read the CPU's counter by default

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Only programs holding a capability may read the CPU's time counter, because precise timers help speculative-execution attacks ([0113](0113-counter-capability.md)). Every other reading of the time is a call to the time service ([0112](0112-time-service.md)), which costs about 530 cycles ([02-verification](../design/02-verification.md)), and std's `Instant::now` makes one for every reading. A thread that counts in memory it shares with another thread also makes a precise timer (Schwarz, Maurice, Gruss and Mangard, 2017), and browsers had to restrict shared memory itself, not only their clocks ([0256](0256-audit-findings-as-evidence.md)). Every program on thisOS can share memory and make threads ([0058](0058-shared-and-handed-over.md), [0083](0083-program-is-address-space.md)). std's `Command` gives a child a copy of the spawner's namespace by default, by handing capabilities over explicitly ([0179](0179-command-copies-namespace.md)).

## Options

1. **On by default,** and withheld by a spawner.
2. **0113 kept,** and judged at checkpoint 4 ([0264](0264-performance-targets.md)).
3. **Readable by every program,** with no capability.

## Decision

Option 1.

- std's `Command` passes the child a copy of the spawner's counter capability by default, as it passes its namespace ([0179](0179-command-copies-namespace.md)). A spawner can leave it out.
- std learns the counter's rate once, from the time service, and `Instant::now` then reads the counter itself.
- Without the capability, `Instant::now` calls the time service, as before.
- Wall-clock time stays a call to the time service ([0112](0112-time-service.md)).
- The kernel still turns access to the counter on and off as it switches programs ([0113](0113-counter-capability.md)).
- This amends 0112, whose monotonic readings no longer always call the time service; 0113, whose capability std now passes by default; and 0179, whose defaults gain it.

## Consequences

- **Reading monotonic time costs a few dozen cycles,** not an IPC round trip.
- **A sandbox can still be denied the counter.** It can still build a slower timer from a counting thread, as any program always could.
- **0173 still holds.** std learns the counter's rate through `svc/time`, so a program without that entry has no clock, whether or not it holds the counter ([0173](0173-time-service-entry.md)).
- **0022 holds.** Nothing is inherited. std hands the capability over explicitly, as 0179 does with the namespace.
- **std needs a counter with a fixed rate,** such as the invariant TSC on x86_64 or the generic timer on AArch64. Where the time service finds none, it tells std to call it instead.
- **Steps 14 and 19 are affected** ([08-roadmap](../design/08-roadmap.md)).
