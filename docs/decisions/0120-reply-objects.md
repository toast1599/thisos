# 0120. Services reply through reply objects

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

A call carries the client's time and priority into the service, and the reply hands them back ([0085](0085-clients-pay-cpu-time.md), [0091](0091-direct-switch-on-calls.md)). A service needs some way to name the call it's answering. Some services answer later than the call arrives, such as a filesystem waiting for a disk.

## Options

1. **Reply objects.** A call gives the service a one-shot reply capability, kept in a reply object the service supplies when it waits, as in seL4's MCS kernel.
2. **One reply per thread.** The reply right sits in the service thread itself, as in classic L4 and seL4, and its next reply uses it.

## Decision

Option 1.

- A service thread names a reply object when it waits on its endpoint.
- When a call arrives, the reply object holds a one-shot capability to answer it.
- Replying through it hands the core, the time and the priority back to the client.
- A service can hold several reply objects, and answer calls in any order.

## Consequences

- **Services can defer replies** without extra threads, for example while they wait for a device.
- **The service pays for its reply objects** ([0026](0026-programs-pay-kernel-objects.md)), and decides how many calls it holds at once. Callers beyond that wait in their own thread objects ([0118](0118-calls-and-notifications.md)).
- **A client is answered exactly once.** A second reply through the same capability fails.
- **A reply can find nobody waiting.** If the client's wait was cancelled ([0134](0134-scheduler-interface.md)) or the client was destroyed, the service's reply fails with an error.
- **Reply objects record whose call a service thread is working on,** which helping needs ([0121](0121-callers-help-busy-services.md)).
- **0085's and 0091's question on the mechanism is answered,** together with [0118](0118-calls-and-notifications.md).
