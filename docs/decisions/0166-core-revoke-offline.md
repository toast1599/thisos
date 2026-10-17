# 0166. Revoking a running core's memory takes the core offline

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

A core's kernel state is made from memory the root task pays for ([0097](0097-user-mode-starts-cores.md), [0165](0165-core-objects.md)). Revoking memory destroys everything made from it, with no cooperation from anyone ([0053](0053-revoke-destroys-derived.md)).

## Options

1. **The core goes offline.**
2. **Revoke is refused while the core runs,** so the root task must stop the core first.

## Decision

Option 1.

- Revoking the memory behind a running core stops the core first. The kernel takes it offline, with PSCI's CPU_OFF on AArch64, or halted with interrupts off on x86_64. It then destroys the core's objects.
- Threads bound to the core don't run until the scheduler binds them to another one.

## Consequences

- **0053 holds for cores.** Revoke needs no cooperation here either.
- **Taking a core offline is a revoke.** To remove a core on purpose, the root task asks the scheduler to move its threads, then revokes.
- **A stopped core can be started again** with new objects ([0165](0165-core-objects.md)).
- **0097's question on revoked core memory is answered.**
