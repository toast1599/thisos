# 0121. A caller that finds its service busy lends it its priority and time

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

A service working on a client's call runs at that client's priority and on its time ([0085](0085-clients-pay-cpu-time.md)). 0085 says this closes priority inversion through services. A review on 2026-10-17 found that this holds only while the service has a thread free.

Suppose a high-priority client calls a service whose only thread is working on a low-priority client's call, at the low priority. Programs at a medium priority can then keep the service from running, and the high-priority client waits without bound. NOVA closes this gap by helping: a caller that finds the service busy runs the service on its own time until the service is free.

## Options

1. **Waiting callers lend.** The kernel runs the busy service thread at the waiting caller's priority, on the caller's time, until it replies.
2. **A floor priority for services.** Each service thread runs at the higher of a floor priority of its own and its caller's priority, as passive servers in seL4's MCS kernel run at their own priority.
3. **Bounded by design.** No mechanism. Services that serve several priorities keep a free thread for each.

## Decision

Option 1.

- When a caller finds the endpoint's receiving thread busy, the kernel runs that thread at the caller's priority, if it's higher, and on the caller's time, until the thread replies.
- If the busy thread is itself waiting on a call to another service, the lending follows that call too.
- So that the kernel knows which thread to raise, each endpoint has at most one receiving thread at a time. A holder of the endpoint's receive right binds it ([0128](0128-rights-and-badges.md)).
- A service that works on several cores at once gives each of its threads an endpoint of its own.
- This amends 0085: while a waiting caller lends, the service works on another client's call on the waiting caller's time.

## Consequences

- **Priority inversion through services is closed,** as 0085 intended, even while the service is busy. A high-priority client waits at most for the work already in progress, which runs at its own priority.
- **The waiting caller pays for that stretch.** It's bounded by one call already in progress, and the caller is the one who gains from it.
- **No low-priority client gains priority by calling a service.** Lending raises a service only while a higher-priority caller is waiting. A floor priority would let a low-priority client flood a high-floor service and take time from the programs in between ([0016](0016-threat-model-scope.md)).
- **Thread pools on one endpoint aren't possible.** Services spread their work across endpoints instead. The rule can be relaxed later without breaking programs.
- **Following calls makes lending a chain.** The kernel walks it with interruption points ([0089](0089-interrupts-off-in-kernel.md)).
- **Lent time appears in the kernel's reports** to the scheduler, charged to the caller that lent it ([0134](0134-scheduler-interface.md)).
