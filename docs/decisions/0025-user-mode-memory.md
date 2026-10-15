# 0025. Physical memory is managed from user mode

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0019](0019-kernel-minimality.md) moves out of the kernel anything that can reasonably leave it. In seL4, the kernel hands out physical memory as capabilities, and user-mode programs carve it into objects and decide how it's used.

## Options

1. **User mode, as in seL4.**
2. **The kernel.** The kernel owns physical memory and allocates it on request.

## Decision

Option 1.

- The kernel hands out physical memory as capabilities and keeps no allocation policy.
- User-mode programs decide how memory is divided and used.

## Consequences

- **All memory starts with the root task** ([0024](0024-root-task.md)), which passes it on.
- **Allocation policy is a user-mode service's job.** For example, how memory is divided between programs ([03-memory](../design/03-memory.md)).
- **Kernel objects are built from this memory** ([0026](0026-programs-pay-kernel-objects.md)).
- **Scarcity is still open.** There are no quotas ([0023](0023-no-quotas.md)), so what a memory service does when demand exceeds what's available is still open ([04-execution](../design/04-execution.md)).
