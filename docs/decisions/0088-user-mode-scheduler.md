# 0088. Scheduling policy runs in user mode

- **Status:** Accepted, amended by [0090](0090-priorities-in-kernel.md)
- **Date:** 2026-10-16

## Context

[01-objective](../design/01-objective.md) left open whether scheduling policy moves out of the kernel ([0019](0019-kernel-minimality.md)). CPU time is shared evenly within each priority ([0033](0033-cpu-fairness.md)), split by account ([0084](0084-shares-belong-to-accounts.md), [0086](0086-priorities-per-program-shares-per-account.md)), and only programs that trust each other share a core ([0034](0034-smt-trusted-pairs.md)).

## Options

1. **Fair queuing in the kernel.** The kernel tracks the time each account, program and thread has used, and runs whoever is furthest behind, as Linux's EEVDF does.
2. **Round robin in the kernel.** Fixed time slices in turn, as in seL4.
3. **A scheduler in user mode.** The kernel only dispatches threads, and a user-mode scheduler picks who runs next.

## Decision

Option 3.

- The kernel only dispatches threads.
- A user-mode scheduler picks which thread runs next.
- Every scheduling choice costs a switch to the scheduler.

## Consequences

- **Scheduling policy can change without changing the kernel** ([0019](0019-kernel-minimality.md)). Fair sharing ([0033](0033-cpu-fairness.md), [0084](0084-shares-belong-to-accounts.md), [0086](0086-priorities-per-program-shares-per-account.md)), thread priorities ([0087](0087-thread-priorities.md)) and pairing on cores ([0034](0034-smt-trusted-pairs.md)) are its policy.
- **The scheduler joins the trusted base.** It decides which programs run, so it can starve any of them, as the memory service can with memory ([0071](0071-three-memory-programs.md)). As an OS service it is bound by [0017](0017-unsafe-code-kernel-and-services.md).
- **Each scheduling choice costs a trip to the scheduler,** on the order of an IPC round trip: about 530 cycles on the development machine ([02-verification](../design/02-verification.md)).
- **Several parts are open** ([04-execution](../design/04-execution.md)): whether strict priorities stay in the kernel, whether calls and replies switch without asking the scheduler ([0085](0085-clients-pay-cpu-time.md)), who records the time each account uses, whether there is one scheduler for each core or one for all, and how the scheduler is started at boot ([06-system](../design/06-system.md)).
- **01's question on scheduling policy is answered.**
