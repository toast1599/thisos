# 0128. Capabilities carry a few rights, and endpoint and notification capabilities carry a badge

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

[0020](0020-capabilities.md) left open what rights a capability carries. Execute permission on memory is already a right ([0054](0054-kernel-enforces-execute.md)). A service needs to tell its clients apart, and the memory service needs to know who pays for a request ([0050](0050-clients-pay.md), [0137](0137-memory-capability-per-program.md)).

## Options

1. **A small mask.** A few rights bits whose meaning depends on the object type, as in seL4.
2. **A right per operation,** as Zircon has, with about 20 rights bits.

## Decision

Option 1.

- Every capability carries a small set of rights bits. What each bit means depends on the type of object:
  - memory: read, write and execute ([0054](0054-kernel-enforces-execute.md));
  - endpoints: send, receive and grant ([0121](0121-callers-help-busy-services.md), [0123](0123-capabilities-in-messages.md));
  - notifications: signal and wait;
  - threads and address spaces: control.
- A copy can only drop rights, never add them.
- Endpoint and notification capabilities also carry a badge: a word set when a capability is copied with one. Once set, it can't be changed.
- A service receives the badge of the capability each message came through. A signal sets its capability's badge bits in the notification.

## Consequences

- **Services tell clients apart by badge.** A service hands each client a capability with a badge of its own.
- **Badges identify payers** to the memory service ([0137](0137-memory-capability-per-program.md)), and **sessions** to the login service ([0141](0141-identity-capability.md)).
- **Several senders can share one notification** and still be told apart, since each sets its own bits ([0122](0122-bound-notifications.md)).
- **The surface to audit stays small.** The kernel checks a few bits on each call.
- **Each object type's exact rights are part of the system library's interface** ([0117](0117-system-library.md)), and grow under its versioning rules ([0143](0143-add-only-versioning.md)).
- **05's question on what rights a capability carries is answered.**
