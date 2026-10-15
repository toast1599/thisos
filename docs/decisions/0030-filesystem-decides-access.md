# 0030. The filesystem service decides who may open stored data

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Capabilities live in memory and don't survive a reboot. Persistent capability systems such as EROS save the capabilities themselves to storage.

## Options

1. **The filesystem service decides**, by its own policy.
2. **Persistent capabilities**, saved to storage and restored at boot.

## Decision

Option 1. After a reboot, the filesystem service decides by its own policy which directory and file capabilities to hand out, for example to a user's login session ([0029](0029-users-as-capabilities.md)).

## Consequences

- **Capabilities don't survive a reboot.** Programs receive fresh ones every boot.
- **Stored data carries access information for that policy.** Its form is designed with the filesystem ([06-system](../design/06-system.md)).
- **The filesystem service guards stored data.** It is trusted with that protection, and it is one of the OS's own services under [0017](0017-unsafe-code-kernel-and-services.md).
