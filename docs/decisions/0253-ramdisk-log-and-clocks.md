# 0253. The ramdisk also holds the log service and the clock drivers

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The ramdisk holds the programs needed to reach storage, and the time and random services they use ([0155](0155-ramdisk-reaches-storage.md)). The log service takes over the second serial port from the root task before storage exists ([0227](0227-steps-and-phases.md), [0236](0236-program-output-second-serial.md)). The time service serves wall-clock time once the clock driver has reported it, and a wall-clock read waits until then ([0252](0252-time-service-starts-early.md)). The clock driver is an ordinary driver, which the device manager starts ([0172](0172-device-manager.md), [0173](0173-time-service-entry.md)). If it came from storage, a filesystem that reads the wall clock while it mounts would wait on itself.

## Options

1. **Both,** the log service and the clock drivers.
2. **The log service only.** The clock drivers come from storage, and no program before storage reads the wall clock.
3. **Neither.** The log service's step designs where early logs go, and the filesystem avoids the wall clock until the clock driver runs.

## Decision

Option 1.

- The ramdisk also holds the log service and the clock drivers.
- This amends 0155, whose list of what the ramdisk holds gains them.

## Consequences

- **Every program that runs before storage has logging and the wall clock.**
- **Reaching storage never waits on storage for the time** ([0252](0252-time-service-starts-early.md)).
- **The ramdisk grows by a few small programs.** It still holds only what the system needs before storage ([0155](0155-ramdisk-reaches-storage.md)).
