# 0036. One IPC message has a small fixed maximum size; larger data goes through shared memory

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Fast IPC carries small messages in registers. [0012](0012-no-memory-cap.md) and [0023](0023-no-quotas.md) rule out limits on what a program may use.

## Options

1. **A small fixed maximum** for one message, with larger data going through shared memory.
2. **No maximum.** The kernel copies large messages itself.

## Decision

Option 1.

- One IPC message has a small fixed maximum size.
- Anything larger goes through shared memory.
- This limits one mechanism, not a program, so it doesn't conflict with 0012 or 0023.

## Consequences

- **The exact size comes with the first IPC design** ([0021](0021-ipc-simple-upgradable.md)). Since IPC is designed to be upgraded, later versions can raise it.
- **Bulk data needs shared memory.** Protocols that move bulk data have to set it up ([03-memory](../design/03-memory.md)).
- **Bindings can hide the split.** Generated bindings can hide it from programs ([0007](0007-abi-calls-and-interfaces.md), Proposed).
