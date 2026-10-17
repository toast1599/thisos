# 0216. A failure of an essential service is handled like a fault in the root task

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

A failed program's supervisor has its memory taken back, and decides on a restart ([0105](0105-supervisor-reclaims.md)). The root task supervises the programs it starts ([0160](0160-root-task-starts-four.md)). The scheduler can't be restarted: only it holds scheduling control ([0161](0161-root-task-before-scheduler.md)), and no thread runs without it. The memory service, the default pager and the IOMMU service hold state that a fresh copy can't rebuild: memory accounts and grants ([0137](0137-memory-capability-per-program.md), [0140](0140-accounts-capability-pair.md)), reservations and mappings ([0138](0138-pager-keeps-reservations.md)), and device mappings and held-back memory ([0170](0170-dma-quarantine.md)). A fault in the root task is logged, then the machine halts or reboots by a boot setting ([0115](0115-root-task-fault-boot-setting.md)).

## Options

1. **Like a fault in the root task.**
2. **Try to restart them.** The root task keeps a copy of scheduling control, and restarts the others from scratch.
3. **The root task's policy,** case by case.

## Decision

Option 1.

- The scheduler, the memory service, the default pager and the IOMMU service are essential services.
- When one fails, the root task logs the failure, then fails the machine through the power capability ([0213](0213-power-capability.md)): it halts or reboots by the boot setting, as after a fault in the root task ([0115](0115-root-task-fault-boot-setting.md)).
- Supervisors restart other services by their own policy, as before ([0105](0105-supervisor-reclaims.md)).

## Consequences

- **The system never runs on without them.** A machine without its memory service or scheduler would hang or misbehave.
- **The root task hears of the scheduler's failure** even with no scheduler running. A fault is handled on the faulting thread's time, as 0085 has for pagers, and switches straight to its handler, as a call does ([0085](0085-clients-pay-cpu-time.md), [0091](0091-direct-switch-on-calls.md)).
- **Production machines come back on their own,** as after a kernel panic ([0110](0110-kernel-panic.md)).
- **Essential services are as critical as the root task.** All four are in the trusted base already ([0071](0071-three-memory-programs.md), [0088](0088-user-mode-scheduler.md), [0169](0169-iommu-service.md)).
