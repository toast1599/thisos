# 0047. The kernel stays mapped in every address space unless the CPU requires otherwise

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Under [0014](0014-microkernel.md), each program has its own address space. On CPUs not affected by Meltdown, the user/kernel split protects a mapped kernel, and Linux keeps it mapped. Unmapping it while user code runs costs two page-table switches per system call, about 316 cycles as measured ([02-verification](../design/02-verification.md)).

## Options

1. **Mapped, unless the CPU needs it unmapped.**
2. **Always unmapped.**

## Decision

Option 1.

- The kernel stays mapped in every address space, reachable only in kernel mode.
- On CPUs affected by Meltdown, it is unmapped while user code runs.

## Consequences

- **No page-table switches on unaffected CPUs.** System calls there pay none.
- **Detection is architecture-specific.** Detecting whether a CPU is affected is architecture-specific code ([0008](0008-architecture-neutral-design.md)).
- **Speculative reads into the kernel rely on the hardware** on unaffected CPUs, through its user/kernel protections ([0016](0016-threat-model-scope.md)).
