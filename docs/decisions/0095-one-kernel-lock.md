# 0095. One lock guards the kernel on multicore

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

On several cores, more than one core can be in the kernel at once. The kernel runs with interrupts off between its interruption points ([0089](0089-interrupts-off-in-kernel.md)), and its code is kept simple enough to prove later ([0018](0018-kernel-assurance.md)).

## Options

1. **One big lock** around the kernel, as in seL4's multicore version.
2. **Fine-grained locks** per object or per core, from the start.
3. **Left to implementation,** like the kernel's layout ([0073](0073-kernel-layout-in-implementation.md)).

## Decision

Option 1.

- One lock guards the whole kernel.
- Finer locking can replace it later without changing any interface.

## Consequences

- **It's the simplest to review and later prove** ([0018](0018-kernel-assurance.md)).
- **Cores contend only while inside the kernel,** which is brief in a microkernel.
- **A core waiting for the lock has to keep answering cross-core requests,** such as TLB flushes, from the core that holds it. Otherwise the two cores wait on each other.
