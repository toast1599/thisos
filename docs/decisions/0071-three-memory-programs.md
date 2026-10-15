# 0071. The root task, the memory service and the default pager are three separate programs

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

The design refers to three roles: the root task, which holds every capability at boot ([0024](0024-root-task.md)); a memory service, which divides memory ([0025](0025-user-mode-memory.md), [0049](0049-memory-fair-share.md)); and a default pager, which most programs are given ([0056](0056-pager-chosen-at-spawn.md)). Which programs fill these roles was undefined.

## Options

1. **One program** for the memory service and the default pager, separate from the root task.
2. **The root task** fills all three roles.
3. **Three programs.**

For faults in the memory service and the default pager:

- **The root task** handles them.
- **Neither faults.** Both keep all their memory mapped.

## Decision

Option 3, with the root task handling their faults.

- The root task, the memory service and the default pager are three separate programs.
- Faults in the memory service and the default pager go to the root task, at the top of the pager chain ([0057](0057-pager-fault-chain.md)).

## Consequences

- **Each program does one job.** The root task keeps boot policy, the memory service divides memory ([0049](0049-memory-fair-share.md), [0062](0062-memory-priority-capability.md)), and the default pager maps memory for the programs that use it.
- **There's no cycle between them.** The default pager depends on the memory service for memory, but neither pages the other.
- **Faults that need memory cross from the pager to the memory service,** one more hop than a combined program would need.
- **All three are in the trusted base,** along with the kernel and the boot path ([0014](0014-microkernel.md)). The root task holds all authority at boot, the memory service decides who gets memory, and the default pager maps memory for most programs. All three are OS services under [0017](0017-unsafe-code-kernel-and-services.md).
- **The root task is a pager as well as boot policy.** How it backs the memory service's and the default pager's memory, and the start sequence, are designed with the boot path ([06-system](../design/06-system.md)).
