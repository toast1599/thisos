# 0112. Programs read the time from a time service

- **Status:** Accepted, amended by [0268](0268-counter-by-default.md)
- **Date:** 2026-10-16

## Context

std's `Instant` and `SystemTime` need a monotonic clock and wall-clock time. The kernel keeps only one timer deadline per core ([0094](0094-one-deadline-per-core.md)), and no record said how programs read the time.

## Options

1. **Read the CPU counter** directly in user mode, and learn its rate once from the system.
2. **Ask the kernel** with a system call.
3. **Ask a time service** in user mode.

## Decision

Option 3. Every reading of the time is a call to a user-mode time service.

## Consequences

- **Every reading costs an IPC round trip,** about 530 cycles on the development machine ([02-verification](../design/02-verification.md)).
- **The kernel gains no clock interface.**
- **The time service is one of the OS's own services** under [0017](0017-unsafe-code-kernel-and-services.md), and ships in the system image ([0044](0044-system-image-is-os-shipped.md)).
- **How programs reach it, and how it reads the hardware clocks,** come with the std mapping and the driver model ([06-system](../design/06-system.md)).
- **Whether programs may still read the CPU's counter themselves** is open ([04-execution](../design/04-execution.md)).
