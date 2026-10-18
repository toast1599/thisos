# 0252. The time service starts early, and serves wall-clock time once the clock driver reports it

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Programs read the time from a time service, through the namespace entry `svc/time` ([0112](0112-time-service.md), [0173](0173-time-service-entry.md)). 0173 has the service read wall-clock time from a clock driver at start. That driver is an ordinary driver, which the device manager starts after hardware discovery ([0172](0172-device-manager.md), [0173](0173-time-service-entry.md)). A program without the entry panics in `Instant::now` ([0173](0173-time-service-entry.md)). The root task starts the scheduler, the memory service, the default pager and the loader, and the loader starts the rest from the boot manifest ([0160](0160-root-task-starts-four.md)).

## Options

1. **Early, with wall-clock time once it's known.**
2. **After the clock driver,** as 0173 reads. Programs started earlier get no `svc/time`.
3. **Early, with wall-clock reads failing** until the driver reports.

## Decision

Option 1.

- The time service starts right after the loader, from the boot manifest ([0160](0160-root-task-starts-four.md)).
- It serves monotonic time from the CPU's counter at once ([0113](0113-counter-capability.md)).
- It serves wall-clock time once the clock driver has reported it. Until then, a wall-clock read waits.
- This amends 0173, whose time service reads wall-clock time when the clock driver reports it, not at its own start.

## Consequences

- **Every program started after the loader can read monotonic time,** the early services included.
- **The clock driver, and what it depends on, never read wall-clock time,** or they would wait on themselves.
- **std's `SystemTime::now` waits during early boot** for programs that ask before the driver reports.
