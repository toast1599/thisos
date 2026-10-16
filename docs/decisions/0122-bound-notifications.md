# 0122. A notification can be bound to a thread

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Each endpoint has one receiving thread at a time ([0121](0121-callers-help-busy-services.md)). That thread waits on its endpoint for calls, but it may also need to hear notifications ([0118](0118-calls-and-notifications.md)), such as a driver's interrupts or a memory notice ([0076](0076-memory-notice-deadline.md)).

## Options

1. **Bound to the thread,** as in seL4.
2. **Separate threads.** A thread waits on one object at a time.
3. **Waiting on a set** of endpoints and notifications at once, like Zircon's ports.

## Decision

Option 1.

- A notification can be bound to one thread.
- While that thread waits on its endpoint, a signal on the bound notification wakes it as if a message had arrived, and hands it the bits.

## Consequences

- **One thread can serve both calls and events.**
- **The kernel adds one field to each thread,** and no wait sets. Helping still knows which thread receives on an endpoint ([0121](0121-callers-help-busy-services.md)).
- **Several event sources can share one notification.** Each sets its own bits through a capability with a different badge ([0128](0128-rights-and-badges.md)).
