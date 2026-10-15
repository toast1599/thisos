# 0026. Programs pay for their kernel objects with memory they hold

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Threads, page tables and capabilities use kernel memory. If the kernel allocated them from its own pool, one program could exhaust that pool, and denial of service is in scope ([0016](0016-threat-model-scope.md)). seL4 avoids this by making programs turn memory they hold into kernel objects.

## Options

1. **The program pays** for its kernel objects with memory it holds.
2. **A kernel pool** from which the kernel allocates its objects.

## Decision

Option 1.

- A kernel object is made from memory that the requesting program holds.
- The kernel keeps no pool of its own for objects that programs create.
- Being limited to the memory a program was handed counts as authority, not as a quota under [0023](0023-no-quotas.md).

## Consequences

- **Kernel memory can't be exhausted.** No program can use up kernel memory that others need.
- **0023 stands as written.** A program may use all the memory it holds, without a cap.
- **The kernel stays smaller.** It needs no allocator for objects that programs create ([0019](0019-kernel-minimality.md)).
- **Reclaiming is still open.** When memory is taken back, the kernel objects made from it have to be destroyed safely. How is open ([03-memory](../design/03-memory.md)).
