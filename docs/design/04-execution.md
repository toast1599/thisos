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

## Open questions

- **What a "program" is:** the unit of isolation, of verification, and of resource accounting. Under [0014](../decisions/0014-microkernel.md), a program is a user-mode process: an address space and its threads. What else it holds, beyond its capabilities ([0020](../decisions/0020-capabilities.md)), is open.
- **Startup information.** Under [0022](../decisions/0022-explicit-spawn.md), a new program receives exactly the capabilities and arguments its creator hands it. Which ones, and in what form ([02-verification](02-verification.md))? They include the program's namespace [0028](../decisions/0028-namespace-at-spawn.md) and its pager [0027](../decisions/0027-user-mode-pagers.md).
- **Threads and scheduling:** threads, scheduler policy, preemption, SMP. Which parts of scheduling live in the kernel ([01-objective](01-objective.md)). Fairness is set by [0033](../decisions/0033-cpu-fairness.md) and core sharing by [0034](../decisions/0034-smt-trusted-pairs.md). Still open: time slices, preemption, SMP, how the scheduler learns which programs trust each other, and priority inversion when a high-priority program waits on a lower-priority service.
- **Fault isolation:** cleaning up after a program that crashes or hangs. Under [0014](../decisions/0014-microkernel.md), the kernel reclaims a failed program's resources. Whether and how failed drivers and services are restarted, as MINIX 3 does, is open.
- **Panic strategy:** unwind or abort. Unwinding needs an unwinder, while aborting pairs with killing the program and cleaning up after it.
- **Resource accounting:** measuring each program's memory, CPU time and handles, without capping them ([0023](../decisions/0023-no-quotas.md)).
- **Memory exhaustion.** Denial of service is in scope ([0016](../decisions/0016-threat-model-scope.md)) and there are no quotas ([0023](../decisions/0023-no-quotas.md)). What happens when one program uses up memory that others need? Under [0025](../decisions/0025-user-mode-memory.md) and [0026](../decisions/0026-programs-pay-kernel-objects.md), a program has only the memory it was handed, so the question becomes how the memory service hands out memory when demand exceeds what's available.
