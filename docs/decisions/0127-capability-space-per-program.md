# 0127. Each program has exactly one capability space

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

To the kernel, a program is an address space and the threads bound to it ([0083](0083-program-is-address-space.md)). 0083 left open whether a program's threads share one capability space. In seL4, each thread names its own.

## Options

1. **One per program.** Each address space is made with one capability space, which all its threads use.
2. **Chosen per thread,** as in seL4.

## Decision

Option 1.

- Each address space is made with exactly one capability space.
- Every thread bound to the address space uses that capability space.

## Consequences

- **The program is the unit of authority,** as it's already the unit of isolation and of fair shares ([0083](0083-program-is-address-space.md), [0084](0084-shares-belong-to-accounts.md)).
- **Threads in one program can't hold different authority.** They share memory and could steer each other anyway.
- **Running part of a program with less authority needs a separate address space,** as 0083 already says.
- **Making a capability space is part of making a program,** so it needs 0031's capability, like the address space ([0031](0031-user-mode-loader.md), [0083](0083-program-is-address-space.md)).
- **A program grows its capability space through its own capability to it** ([0108](0108-startup-information.md), [0126](0126-capability-space-radix-table.md)).
- **0083's question on capability spaces is answered.**
