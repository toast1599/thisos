# 0140. An account is a pair of capabilities, one from the scheduler and one from the memory service

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Fair shares of CPU time and memory belong to accounts ([0084](0084-shares-belong-to-accounts.md)). The scheduler keeps the CPU accounts, and the memory service keeps the memory accounts ([0092](0092-scheduler-keeps-time.md)). A new program joins its spawner's account, unless a holder of an account-creation capability gives it a new one ([0084](0084-shares-belong-to-accounts.md)).

## Options

1. **A pair of capabilities,** one from each service.
2. **One account service,** which makes accounts and tells the scheduler and the memory service about them.

## Decision

Option 1.

- An account is a CPU account in the scheduler and a memory account in the memory service. Each half is named by a capability its service issues.
- The right to make accounts is also a pair of capabilities, one for each service.
- A spawner that places a program in an account passes both capabilities. The loader presents them when it builds the program ([0134](0134-scheduler-interface.md), [0137](0137-memory-capability-per-program.md)).

## Consequences

- **No program joins the trusted base** ([0071](0071-three-memory-programs.md), [0088](0088-user-mode-scheduler.md)).
- **A holder of two accounts could mix their halves.** That affects only the shares of accounts it already holds.
- **The login service gets the right to make accounts at boot,** from the root task. How the root task gets it from the two services is designed with the boot path ([06-system](../design/06-system.md)), like the right to grant memory priorities ([0062](0062-memory-priority-capability.md)).
- **0084's question on how accounts are made is answered.**
