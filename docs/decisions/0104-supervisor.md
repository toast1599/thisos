# 0104. Every program has a supervisor, named at spawn, that learns how it ended

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

std's `Child::wait` needs an exit status, and someone has to learn when a program crashes or is stopped to free memory ([0049](0049-memory-fair-share.md)). [04-execution](../design/04-execution.md) left open who is told.

## Options

1. **A supervisor named at spawn,** normally the spawner.
2. **Holders of a program capability.** The kernel signals them when the program's address space is destroyed, and keeps a status word for them, as Zircon does.
3. **The loader,** which tells the spawners.

## Decision

Option 1.

- Every program has a supervisor, normally its spawner.
- A program reports its own exit status to its supervisor.
- The supervisor is told when the program crashes or is stopped.
- All of this happens in user mode. The kernel adds nothing.

## Consequences

- **The supervisor is part of a program's startup information** ([04-execution](../design/04-execution.md)).
- **How the supervisor is told** comes with the interfaces ([05-interfaces](../design/05-interfaces.md)). The program's exception handler ([0102](0102-exception-handler.md)) sees crashes, and the memory service stops programs to free memory ([0049](0049-memory-fair-share.md)).
- **std's `Child::wait` maps onto this report** ([06-system](../design/06-system.md)).
- **04's question on who is told when a program is stopped to free memory is answered.**
