# 04. Programs, isolation and scheduling

- **Day:** 4 (Sat 2026-10-17)
- **Status:** Not started

## Covers

What a program is, how programs run and are scheduled, and what happens when one fails.

## Decided

- [0022](../decisions/0022-explicit-spawn.md) Programs start other programs by explicit spawn
- [0023](../decisions/0023-no-quotas.md) No quotas: a program's resources are limited only by the hardware and what's available

## Open questions

- **What a "program" is:** the unit of isolation, of verification, and of resource accounting. Under [0014](../decisions/0014-microkernel.md), a program is a user-mode process: an address space and its threads. What else it holds, beyond its capabilities ([0020](../decisions/0020-capabilities.md)), is open.
- **Startup information.** Under [0022](../decisions/0022-explicit-spawn.md), a new program receives exactly the capabilities and arguments its creator hands it. Which ones, and in what form ([02-verification](02-verification.md))?
- **Threads and scheduling:** threads, scheduler policy, preemption, SMP. Which parts of scheduling live in the kernel ([01-objective](01-objective.md)), and how the scheduler keeps one program from starving the others ([0016](../decisions/0016-threat-model-scope.md)).
- **Fault isolation:** cleaning up after a program that crashes or hangs. Under [0014](../decisions/0014-microkernel.md), the kernel reclaims a failed program's resources. Whether and how failed drivers and services are restarted, as MINIX 3 does, is open.
- **Panic strategy:** unwind or abort. Unwinding needs an unwinder, while aborting pairs with killing the program and cleaning up after it.
- **Resource accounting:** measuring each program's memory, CPU time and handles, without capping them ([0023](../decisions/0023-no-quotas.md)).
- **Memory exhaustion.** Denial of service is in scope ([0016](../decisions/0016-threat-model-scope.md)) and there are no quotas ([0023](../decisions/0023-no-quotas.md)). What happens when one program uses up memory that others need?
