# 04. Programs, isolation and scheduling

- **Day:** 4 (Fri 2026-10-16)
- **Status:** Frozen

## Covers

What a program is, how programs run and get scheduled, and what happens when one fails.

## Decided

- [0022](../decisions/0022-explicit-spawn.md) Programs start other programs by explicit spawn
- [0023](../decisions/0023-no-quotas.md) No quotas: a program's resources are limited only by the hardware and what's available
- [0031](../decisions/0031-user-mode-loader.md) The program loader runs in user mode, and starting programs needs a capability
- [0033](../decisions/0033-cpu-fairness.md) CPU fairness: fair sharing within each priority, and priorities as capabilities (amended by [0084](../decisions/0084-shares-belong-to-accounts.md))
- [0034](../decisions/0034-smt-trusted-pairs.md) Only programs that trust each other share a CPU core
- [0035](../decisions/0035-predictor-flush-on-request.md) Branch predictors are flushed only on request (superseded by [0080](../decisions/0080-predictor-flush-both-ways.md))
- [0049](../decisions/0049-memory-fair-share.md) When memory runs short, it's shared fairly within each memory priority (amended by [0084](../decisions/0084-shares-belong-to-accounts.md))
- [0056](../decisions/0056-pager-chosen-at-spawn.md) A program's pager is chosen when it's spawned (amended by [0081](../decisions/0081-pager-per-thread.md))
- [0062](../decisions/0062-memory-priority-capability.md) Memory priority is a capability of its own
- [0080](../decisions/0080-predictor-flush-both-ways.md) Branch predictors are flushed when switching to or away from a program that asks
- [0081](../decisions/0081-pager-per-thread.md) Page faults go to the faulting thread's pager
- [0082](../decisions/0082-corrections.md) Corrections to 0003, 0006, 0008, 0014 and 0077
- [0083](../decisions/0083-program-is-address-space.md) To the kernel, a program is an address space and the threads bound to it
- [0084](../decisions/0084-shares-belong-to-accounts.md) Fair shares of CPU time and memory belong to accounts (amended by [0092](../decisions/0092-scheduler-keeps-time.md))
- [0085](../decisions/0085-clients-pay-cpu-time.md) Clients pay for the CPU time services spend on them (amended by [0121](../decisions/0121-callers-help-busy-services.md))
- [0086](../decisions/0086-priorities-per-program-shares-per-account.md) Priorities stay per program, and each priority is shared by account
- [0087](../decisions/0087-thread-priorities.md) A thread runs at any CPU priority up to its program's
- [0088](../decisions/0088-user-mode-scheduler.md) Scheduling policy runs in user mode (amended by [0090](../decisions/0090-priorities-in-kernel.md))
- [0089](../decisions/0089-interrupts-off-in-kernel.md) The kernel runs with interrupts off between its interruption points (amended by [0218](../decisions/0218-lock-yield.md))
- [0090](../decisions/0090-priorities-in-kernel.md) Strict priorities stay in the kernel, and the scheduler orders threads within each priority
- [0091](../decisions/0091-direct-switch-on-calls.md) Calls and replies switch directly between client and service (amended by [0219](../decisions/0219-wakeup-preemption.md))
- [0092](../decisions/0092-scheduler-keeps-time.md) The scheduler keeps the CPU time books, from the kernel's reports
- [0093](../decisions/0093-scheduler-per-core.md) The scheduler has a thread on each core, which decides for that core
- [0094](../decisions/0094-one-deadline-per-core.md) Each core has one timer deadline, set by its scheduler thread
- [0095](../decisions/0095-one-kernel-lock.md) One lock guards the kernel on multicore
- [0097](../decisions/0097-user-mode-starts-cores.md) User mode finds the CPUs, and the root task asks the kernel to start each one
- [0098](../decisions/0098-core-sharing-capability.md) Programs share a core only if both hold the same core-sharing capability
- [0099](../decisions/0099-kernel-beside-sibling.md) The kernel doesn't pause a sibling hardware thread when it runs
- [0100](../decisions/0100-cpu-flaw-mitigations.md) The kernel applies each CPU's documented mitigations for speculative-execution flaws
- [0101](../decisions/0101-eager-fpu-state.md) Floating-point and vector registers are saved and restored on every switch
- [0102](../decisions/0102-exception-handler.md) Each thread names an exception handler besides its pager
- [0103](../decisions/0103-fault-without-handler.md) A fault with nowhere to go suspends the thread, and in the root task halts the system (amended by [0115](../decisions/0115-root-task-fault-boot-setting.md))
- [0104](../decisions/0104-supervisor.md) Every program has a supervisor, named at spawn, that learns how it ended
- [0105](../decisions/0105-supervisor-reclaims.md) A failed program's supervisor has its memory taken back, and decides on a restart
- [0106](../decisions/0106-pager-passes-on-faults.md) A pager passes faults it won't resolve to the thread's exception handler
- [0107](../decisions/0107-loader-writes-startup-data.md) The loader writes a new program's startup information into its memory (amended by [0160](../decisions/0160-root-task-starts-four.md))
- [0108](../decisions/0108-startup-information.md) What a new program receives when it's spawned (amended by [0146](../decisions/0146-startup-data-block.md), [0174](../decisions/0174-random-service-and-seed.md), [0176](../decisions/0176-paths-and-current-directory.md) and [0179](../decisions/0179-command-copies-namespace.md))
- [0109](../decisions/0109-panic-strategy.md) Third-party programs unwind on panic by default, and the OS's own services abort
- [0110](../decisions/0110-kernel-panic.md) A kernel panic is logged, then the machine halts or reboots by a boot setting
- [0111](../decisions/0111-resource-accounting.md) Each service reports the resource use it already tracks
- [0112](../decisions/0112-time-service.md) Programs read the time from a time service
- [0113](../decisions/0113-counter-capability.md) Only programs holding a capability may read the CPU's time counter
- [0114](../decisions/0114-scheduler-priority.md) Each core's scheduler thread runs above every priority it schedules
- [0115](../decisions/0115-root-task-fault-boot-setting.md) A fault in the root task is handled like a kernel panic

The final review ([07-engineering](07-engineering.md)) added these:

- [0213](../decisions/0213-power-capability.md) A power capability lets its holder reboot, power off or fail the machine
- [0216](../decisions/0216-essential-service-failure.md) A failure of an essential service is handled like a fault in the root task
- [0218](../decisions/0218-lock-yield.md) Long kernel operations also yield the kernel lock at their interruption points
- [0219](../decisions/0219-wakeup-preemption.md) A wakeup enters the scheduler only if the woken thread may run before the running one
- [0220](../decisions/0220-endpoint-priority-order.md) Callers waiting on an endpoint are served by priority, then arrival

And Day 8 ([08-roadmap](08-roadmap.md)) added two more:

- [0246](../decisions/0246-interruption-stretch-limit.md) Work between interruption points is limited in code, and no stretch may take more than 20 µs
- [0252](../decisions/0252-time-service-starts-early.md) The time service starts early, and serves wall-clock time once the clock driver reports it

## Open questions

None are left in this document.
