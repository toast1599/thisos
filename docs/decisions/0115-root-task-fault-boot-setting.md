# 0115. A fault in the root task is handled like a kernel panic

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

A fault in the root task halts the system with a log message ([0103](0103-fault-without-handler.md)), while a kernel panic halts or reboots according to a boot setting ([0110](0110-kernel-panic.md)). A production machine whose root task faulted would stay down.

## Options

1. **Follow the boot setting.**
2. **Always halt,** as 0103 says.

## Decision

Option 1.

- A fault in the root task is logged, then the machine halts or reboots by the same boot setting as a kernel panic.
- This amends 0103, whose root-task faults no longer always halt.

## Consequences

- **Both failures of the trusted base behave alike** ([0024](0024-root-task.md)).
- **Production machines come back on their own** after either.
