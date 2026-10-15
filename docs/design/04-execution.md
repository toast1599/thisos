# 04. Programs, isolation and scheduling

- **Day:** 4 (Sat 2026-10-17)
- **Status:** Not started

## Covers

What a program is, how programs run and are scheduled, and what happens when one fails.

## Decided

- [0022](../decisions/0022-explicit-spawn.md) Programs start other programs by explicit spawn
- [0023](../decisions/0023-no-quotas.md) No quotas: a program's resources are limited only by the hardware and what's available
- [0031](../decisions/0031-user-mode-loader.md) The program loader runs in user mode, and starting programs needs a capability
- [0033](../decisions/0033-cpu-fairness.md) CPU fairness: fair sharing within each priority, and priorities as capabilities
- [0034](../decisions/0034-smt-trusted-pairs.md) Only programs that trust each other share a CPU core
- [0035](../decisions/0035-predictor-flush-on-request.md) Branch predictors are flushed only on request
- [0049](../decisions/0049-memory-fair-share.md) When memory runs short, it's shared fairly within each memory priority
- [0056](../decisions/0056-pager-chosen-at-spawn.md) A program's pager is chosen when it's spawned
- [0062](../decisions/0062-memory-priority-capability.md) Memory priority is a capability of its own

## Open questions

- **What a "program" is:** the unit of isolation, of verification, and of resource accounting. Under [0014](../decisions/0014-microkernel.md), a program is a user-mode process: an address space and its threads. What else it holds, beyond its capabilities ([0020](../decisions/0020-capabilities.md)), is open.
- **Startup information.** Under [0022](../decisions/0022-explicit-spawn.md), a new program receives exactly the capabilities and arguments its creator hands it. Which ones, and in what form ([02-verification](02-verification.md))? They include the program's namespace [0028](../decisions/0028-namespace-at-spawn.md), its pager [0027](../decisions/0027-user-mode-pagers.md), named by the spawner and normally the default pager [0056](../decisions/0056-pager-chosen-at-spawn.md), [0071](../decisions/0071-three-memory-programs.md), and its CPU and memory priorities [0033](../decisions/0033-cpu-fairness.md), [0062](../decisions/0062-memory-priority-capability.md).
- **Threads and scheduling:** threads, scheduler policy, preemption, SMP. Which parts of scheduling live in the kernel ([01-objective](01-objective.md)). Fairness is set by [0033](../decisions/0033-cpu-fairness.md) and core sharing by [0034](../decisions/0034-smt-trusted-pairs.md). Still open: time slices, preemption, SMP, how the scheduler learns which programs trust each other, priority inversion when a high-priority program waits on a lower-priority service, and whether the kernel runs with interrupts off between interruption points [0060](../decisions/0060-kernel-stack-per-core.md).
- **Fault isolation:** cleaning up after a program that crashes or hangs. Under [0053](../decisions/0053-revoke-destroys-derived.md), revoking the memory a failed program was granted destroys everything made from it. Who notices the failure and has the memory service take that memory back is open. Whether and how failed drivers and services are restarted, as MINIX 3 does, is open. So is who is told when a program is stopped to free memory [0049](../decisions/0049-memory-fair-share.md).
- **Panic strategy:** unwind or abort. Unwinding needs an unwinder, while aborting pairs with killing the program and cleaning up after it.
- **Resource accounting:** measuring each program's memory, CPU time and handles, without capping them ([0023](../decisions/0023-no-quotas.md)).
- **What counts as one share.** CPU time [0033](../decisions/0033-cpu-fairness.md) and memory [0049](../decisions/0049-memory-fair-share.md) are both shared fairly. Are shares counted per program, or along the spawn tree, so that a program can't enlarge its share by spawning children?
