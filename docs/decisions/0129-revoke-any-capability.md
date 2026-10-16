# 0129. Revoking any capability destroys every copy made from it

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Revoking memory destroys everything made from it ([0053](0053-revoke-destroys-derived.md)). Capabilities that aren't memory get copied too: in messages ([0123](0123-capabilities-in-messages.md)) and into the namespaces of new programs ([0131](0131-namespace-path-prefixes.md)). A service may need to cut off one client, and a spawner to take back what it passed on.

## Options

1. **Any capability,** as in seL4.
2. **Memory only.** Other capabilities are deleted one by one by whoever holds them.

## Decision

Option 1.

- Revoking any capability destroys every copy made from it, wherever the copies went, along with everything made from those copies.
- The revoked capability itself stays.
- Revoking uses the records the kernel already keeps under 0053, and runs in steps that can be interrupted.

## Consequences

- **A service can cut off one client** by revoking the badged capability it gave that client ([0128](0128-rights-and-badges.md)). Every other client keeps working.
- **A spawner can take back what it passed** to a program it started.
- **Revoke needs no cooperation** from the holders of the copies, as 0053 already holds for memory.
- **Deleting one capability leaves its copies alone.** Only revoking reaches them.
- **05's question on how capabilities are revoked is answered.**
