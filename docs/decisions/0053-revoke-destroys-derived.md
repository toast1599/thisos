# 0053. Revoking memory destroys everything made from it

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0026](0026-programs-pay-kernel-objects.md) left open how kernel objects are destroyed safely when the memory they were made from is taken back. [0049](0049-memory-fair-share.md) needs to take memory back by force.

## Options

1. **Revoke destroys all.** The kernel tracks what was made from which memory, and revoking destroys all of it.
2. **Holder frees first.** Memory can be taken back only after its holder has destroyed what it made from it.

## Decision

Option 1.

- The kernel records which objects and capabilities were made from which memory, as seL4 does.
- Revoking memory destroys every object and capability made from it, and removes every mapping of it.
- A revoke runs in steps that can be interrupted, so a large one doesn't stall the system.

## Consequences

- **Memory can be taken back from any program,** whether it cooperates or not. [0049](0049-memory-fair-share.md) relies on this.
- **Revoked memory is cleared before it's used again,** because the kernel clears memory before it's made into anything new ([0051](0051-kernel-clears-memory.md)).
- **Programs using the memory get errors, not a crash** ([0065](0065-lost-memory-is-an-error.md)).
- **The kernel keeps records of what was made from what.** They're part of the capability design ([05-interfaces](../design/05-interfaces.md)), and programs pay for the space they take ([0026](0026-programs-pay-kernel-objects.md)).
- **Revoking costs TLB flushes** on the cores where the affected address spaces ran.
- **Long operations need points where they can be interrupted,** which fits one kernel stack per core ([0060](0060-kernel-stack-per-core.md)).
- **03's reclaiming question is answered.**
