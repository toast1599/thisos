# 0092. The scheduler keeps the CPU time books, from the kernel's reports

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

[0084](0084-shares-belong-to-accounts.md) gave the kernel an account object for CPU time. Under [0088](0088-user-mode-scheduler.md) a user-mode scheduler decides, but only the kernel sees every switch, including the direct ones on calls and replies ([0091](0091-direct-switch-on-calls.md)).

## Options

1. **The kernel charges accounts.** It keeps 0084's account objects and charges them as time is spent, and the scheduler reads them.
2. **The scheduler keeps them.** The kernel reports how long each thread ran, and the scheduler charges accounts itself.

## Decision

Option 2.

- The kernel reports how long each thread ran, including time in direct switches on calls and replies ([0091](0091-direct-switch-on-calls.md)).
- The scheduler charges accounts itself.
- This amends 0084: the kernel has no account object for CPU time.

## Consequences

- **Accounts live in user mode.** CPU accounts are the scheduler's, and memory accounts are the memory service's ([0025](0025-user-mode-memory.md), [0084](0084-shares-belong-to-accounts.md)).
- **The kernel gains no object type for accounts** ([0019](0019-kernel-minimality.md)).
- **Reports have to say whose time a thread ran on,** so that a service's time on a call is charged to its client ([0085](0085-clients-pay-cpu-time.md)).
- **How reports reach the scheduler within registers** ([0072](0072-registers-only.md)), including chains of calls, comes with the interfaces ([05-interfaces](../design/05-interfaces.md)).
