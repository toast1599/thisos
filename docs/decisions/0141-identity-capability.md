# 0141. The login service gives each session an identity capability

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

A user is the set of capabilities a login service grants, and the kernel has no notion of users ([0029](0029-users-as-capabilities.md)). Features that conventionally rely on identity, such as audit logs and file ownership, need a design based on capabilities ([0029](0029-users-as-capabilities.md)). After a reboot, the filesystem decides by its own policy which capabilities to hand a user's session ([0030](0030-filesystem-decides-access.md)).

## Options

1. **An identity capability,** which services can check with the login service.
2. **Badges in logs.** Each service logs the badge a request came through, and the login service records which badges it gave which user.

## Decision

Option 1.

- The login service gives each session an identity capability: a badged capability to the login service itself ([0128](0128-rights-and-badges.md)).
- A program presents it to a service, which asks the login service whom it names and records the answer.
- It proves who is acting, but it grants no access by itself. Access still comes only from the capabilities a program holds ([0020](0020-capabilities.md)).
- Services use it for their records. Only services that decide access by their own policy may act on it, such as the filesystem when it chooses what to hand a session after a reboot ([0030](0030-filesystem-decides-access.md)).

## Consequences

- **Audit logs and file ownership can name users,** without the kernel or ordinary services knowing about users.
- **Identity doesn't become ambient authority.** Acting on it is limited to services that decide access by their own policy.
- **A program can choose not to identify itself,** and then acts only through its capabilities.
- **The login service gains a call that answers whom an identity capability names.** It's trusted with that, as it's already trusted with keeping users apart ([0029](0029-users-as-capabilities.md)).
- **What the filesystem stores about owners** is designed with stored access information ([06-system](../design/06-system.md)).
- **05's question on identity without user IDs is answered.**
