# 0029. A user is the set of capabilities a login service grants

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0001](0001-general-purpose-os.md) requires multiple users, and capabilities ([0020](0020-capabilities.md)) replace user IDs.

## Options

1. **Capabilities from a login service.** A user is the set of capabilities that service grants.
2. **An identity the kernel knows.** The kernel tags each program with a user identity that services can check.

## Decision

Option 1.

- A user is the set of capabilities that a login service grants after authentication.
- The kernel has no notion of users.

## Consequences

- **Users are kept apart by their capabilities.** Separation between users is separation between the capabilities they hold, enforced like any other access.
- **The login service decides who gets what.** That makes it trusted with keeping users apart. It is one of the OS's own services under [0017](0017-unsafe-code-kernel-and-services.md).
- **Identity-based features need new designs.** Anything that conventionally relies on identity, such as audit logs or file ownership, needs a design based on capabilities ([05-interfaces](../design/05-interfaces.md)).
