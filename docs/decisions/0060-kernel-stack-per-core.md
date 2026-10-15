# 0060. The kernel has one stack per CPU core, and its worst-case depth is checked at build time

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Safe Rust detects stack overflow only at run time. [03-memory](../design/03-memory.md) listed what the kernel needs in any case: guard pages, stack probes in the target spec, and fault handlers that run on their own stacks. seL4 has one kernel stack per core. Most kernels have one per thread.

## Options

1. **One per core, checked.** Each core has one kernel stack, and the build checks the kernel's worst-case stack depth.
2. **One per thread,** paid for by the thread's program.

## Decision

Option 1.

- Each CPU core has one kernel stack.
- The kernel never blocks part-way through an operation. Long operations, such as a large revoke ([0053](0053-revoke-destroys-derived.md)) or clearing a large piece of memory ([0051](0051-kernel-clears-memory.md)), stop at set points where they can be interrupted.
- The kernel uses no recursion, and the build checks its worst-case stack depth.
- Guard pages, stack probes, and fault handlers on their own stacks apply as well.

## Consequences

- **Thread objects stay small,** because they carry no kernel stack. Programs pay less for each thread ([0026](0026-programs-pay-kernel-objects.md)).
- **Kernel stack use is bounded and known,** not just guarded.
- **Kernel code is written around interruption points.** Nothing in the kernel waits in the middle of an operation.
- **The depth check needs a tool.** Which one is open ([07-engineering](../design/07-engineering.md)).
- **Fault stacks are architecture-specific** ([0008](0008-architecture-neutral-design.md)). On x86_64 they use the interrupt stack table.
- **Whether the kernel runs with interrupts off between interruption points,** as seL4 does, is open ([04-execution](../design/04-execution.md)).
- **03's stack overflow question is answered.**
