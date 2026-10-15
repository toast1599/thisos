# 0062. Memory priority is a capability of its own

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Under [0049](0049-memory-fair-share.md), memory is shared fairly within each memory priority when it runs short, and priorities are strict. CPU priority is already a capability ([0033](0033-cpu-fairness.md)).

## Options

1. **Its own capability,** separate from CPU priority.
2. **The same as CPU priority.** One priority capability governs both.

## Decision

Option 1.

- Memory priority is a capability separate from CPU priority.
- A program has the memory priority it was granted, and can never raise it itself.
- The memory service checks memory priority. The kernel has no memory policy ([0025](0025-user-mode-memory.md)).

## Consequences

- **Granting one doesn't grant the other.** Protecting a database's memory doesn't let it starve others of CPU time, and the reverse holds too.
- **Granting a high memory priority is a trust decision,** because priorities are strict and it lets a program cause lower-priority programs to be stopped ([0049](0049-memory-fair-share.md)).
- **The memory service issues them, not the kernel,** so the root task doesn't receive them at boot as it does kernel capabilities ([0024](0024-root-task.md)). How the right to grant them reaches the root task is designed with the boot path ([06-system](../design/06-system.md)).
- **There are two priorities to grant per program,** which spawners and the login service ([0029](0029-users-as-capabilities.md)) have to manage.
