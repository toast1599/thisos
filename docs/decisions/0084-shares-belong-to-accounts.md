# 0084. Fair shares of CPU time and memory belong to accounts

- **Status:** Accepted, amended by [0092](0092-scheduler-keeps-time.md)
- **Date:** 2026-10-16

## Context

CPU time is shared evenly between programs at the same priority ([0033](0033-cpu-fairness.md)), and memory is shared fairly within each memory priority when it runs short ([0049](0049-memory-fair-share.md)). [04-execution](../design/04-execution.md) asked whether shares are counted per program or along the spawn tree. If every program were one share, a program could enlarge its share by spawning, and a user running more programs would get more than one running fewer. The system has to serve several users ([0001](0001-general-purpose-os.md), [0029](0029-users-as-capabilities.md)).

## Options

1. **Per program.** Each address space ([0083](0083-program-is-address-space.md)) is one share.
2. **Accounts.** A share belongs to an account that a program joins when it's spawned.
3. **Spawn tree.** Each program's share is nested inside its spawner's, and siblings split their parent's share evenly.

## Decision

Option 2.

- A fair share of CPU time and of memory belongs to an account, not to a single program.
- A new program joins its spawner's account, unless a holder of an account-creation capability gives it a new one. The login service is an example, with one account per session.
- The programs in an account split its share evenly.
- This amends 0033, whose CPU time is shared evenly between accounts at the same priority; 0049, whose fair shares are per account; and [0064](0064-memory-belongs-to-payer.md), whose memory counts against the account of the program it was granted to.

## Consequences

- **Spawning doesn't enlarge a share.** A program's children share its account unless a holder of the capability decides otherwise.
- **Users can't outweigh each other by running more programs,** when each session has its own account.
- **Granting a new account is a trust decision,** like granting a priority ([0033](0033-cpu-fairness.md), [0062](0062-memory-priority-capability.md)).
- **The kernel gains an account object for CPU time,** paid for like other kernel objects ([0026](0026-programs-pay-kernel-objects.md)). Memory accounts are kept by the memory service, which holds memory policy ([0025](0025-user-mode-memory.md)).
- **CPU time is shared at two levels:** between accounts, then between the programs in each.
- **The payer doesn't change.** Memory still belongs to the program it was granted to, which can always take it back ([0064](0064-memory-belongs-to-payer.md)). Only the share it counts against is the account's.
- **How accounts are made, and how the capability reaches the login service,** come with the interfaces ([05-interfaces](../design/05-interfaces.md)) and the boot path ([06-system](../design/06-system.md)).
- **04's question on what counts as one share is answered.**
