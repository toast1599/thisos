# 0085. Clients pay for the CPU time services spend on them

- **Status:** Accepted, amended by [0121](0121-callers-help-busy-services.md)
- **Date:** 2026-10-16

## Context

Clients already pay for the memory services keep for them ([0050](0050-clients-pay.md)). CPU time had no such rule. A service spends its own share on its clients' calls, and a pager spends its own share on faults ([0027](0027-user-mode-pagers.md)), so one client that floods a service with calls can starve its other clients. Denial of service is in scope ([0016](0016-threat-model-scope.md)). A high-priority program that waits on a lower-priority service also suffers priority inversion, which [0033](0033-cpu-fairness.md) left open.

## Options

1. **Clients pay.** A call carries the caller's time and priority into the service while it works on it, as with passive servers in seL4 MCS.
2. **Priority inheritance only.** Services run on their own share, but while serving a client they run at least at its priority, as in QNX.
3. **Services' own time.** Services run at their own priority on their own share, and each is fair to its clients by its own policy.

## Decision

Option 1.

- While a service works on a client's call, it runs on the client's time and at the client's priority.
- A pager handles a fault on the faulting program's time.

## Consequences

- **No client can exhaust a service's CPU time.** A client that floods a service pays for the flood from its own account ([0084](0084-shares-belong-to-accounts.md)).
- **Priority inversion through services is closed.** A service works for a high-priority client at that client's priority. This answers 0033's open question.
- **No cap is involved.** The time is charged to the client's share, not limited by a budget, so [0023](0023-no-quotas.md) holds.
- **Services have to cope with a client's time running out** in the middle of a request.
- **The mechanism comes with the first IPC design** ([0021](0021-ipc-simple-upgradable.md), [05-interfaces](../design/05-interfaces.md)). It needs calls that the service replies to, so the time can travel with the call and return with the reply.
