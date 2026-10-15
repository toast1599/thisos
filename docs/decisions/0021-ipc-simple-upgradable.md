# 0021. IPC starts simple and modular, and is designed to be upgraded

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Under the microkernel ([0014](0014-microkernel.md)), almost everything goes through IPC, so its cost largely decides performance. A round trip costs at least about 530 cycles, estimated in [02-verification](../design/02-verification.md). There were two candidate models:
- synchronous calls with shared memory, as in L4 and seL4;
- asynchronous channels, as in Zircon.

## Options

1. Synchronous calls, shared memory and notifications.
2. Asynchronous channels.

## Decision

- IPC starts with a simple design, built from separate parts.
- It's designed so it can be upgraded later without breaking programs.

## Consequences

- **The first design comes on Day 5.** Which simple design comes first is decided then ([05-interfaces](../design/05-interfaces.md)).
- **IPC interfaces are versioned.** Upgrades must not break programs, so they're versioned like every other interface ([0007](0007-abi-calls-and-interfaces.md), Proposed).
- **Programs must not depend on the mechanism.** They should reach IPC in a way that lets the mechanism underneath change. How (generated bindings, a library layer) is open ([05-interfaces](../design/05-interfaces.md)).
- **Speed comes later, where it's measured.** Faster paths can be added where measurements show they're needed.
