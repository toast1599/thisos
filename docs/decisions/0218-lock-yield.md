# 0218. Long kernel operations also yield the kernel lock at their interruption points

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

One lock guards the kernel on multicore ([0095](0095-one-kernel-lock.md)). The kernel runs with interrupts off, and long operations, such as a large revoke ([0053](0053-revoke-destroys-derived.md)) or clearing a large piece of memory ([0051](0051-kernel-clears-memory.md)), check for pending interrupts at their interruption points ([0089](0089-interrupts-off-in-kernel.md)). A pending interrupt arrives only on the operation's own core. So a program that revokes a large region holds the lock for the whole operation, while every other core waits to enter the kernel.

## Options

1. **Yield at interruption points.**
2. **Accept it.**
3. **Finer locks now,** reversing 0095.

## Decision

Option 1.

- At each interruption point, a long operation also stops when another core is waiting for the kernel lock. It continues later, as after an interrupt.
- This amends 0089, whose interruption points now check for a waiting core as well as for pending interrupts.

## Consequences

- **No core waits for the lock longer than the longest stretch between interruption points,** which Day 8 sets ([0089](0089-interrupts-off-in-kernel.md), [08-roadmap](../design/08-roadmap.md)).
- **One lock stays simple to review** ([0095](0095-one-kernel-lock.md)), without letting one program stall the others.
- **Long operations take longer when cores contend,** since they stop more often.
