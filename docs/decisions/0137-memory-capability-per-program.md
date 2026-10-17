# 0137. Each program has its own capability to the memory service, which names it as payer

- **Status:** Accepted, amended by [0164](0164-root-task-memory-pool.md)
- **Date:** 2026-10-17

## Context

Memory belongs to the program it was granted to, its payer ([0064](0064-memory-belongs-to-payer.md)), and counts against that program's account ([0084](0084-shares-belong-to-accounts.md)). Requests for a program's memory don't all come from the program itself. Its pager backs its pages and grows its stacks on faults ([0048](0048-reserve-on-grant.md), [0081](0081-pager-per-thread.md)), and the loader builds its address space and page tables from its memory ([0052](0052-user-mode-page-tables.md)). Badges tell a service which capability a request came through ([0128](0128-rights-and-badges.md)).

## Options

1. **A badged capability for each program,** held by the program, its pager and the loader.
2. **Only the program asks,** then lends the memory to its pager and the loader ([0050](0050-clients-pay.md)).

## Decision

Option 1.

- Each program has its own capability to the memory service, with a badge that names the program.
- Memory requested through it is granted to that program and counts against its account, whoever makes the request.
- The program's pager and the loader hold copies, so they can back its pages, make its page tables and grow its stacks.
- The memory service makes the capability when a spawner asks. The new program joins the spawner's account, or an account the spawner holds the capabilities for ([0140](0140-accounts-capability-pair.md)).
- Along with it, the spawner receives the reclaim capability that the program's supervisor uses ([0105](0105-supervisor-reclaims.md)).

## Consequences

- **Stacks can grow on faults,** as 0048 requires, because the pager can get memory for the program at fault time. Under option 2, a stack could grow only into memory lent in advance.
- **The payer never depends on who asks.** Memory requested by a pager or the loader counts against the program it was requested for ([0064](0064-memory-belongs-to-payer.md)).
- **Whoever holds a copy can spend the program's memory.** Those are its pager and the loader, which are trusted with its address space anyway.
- **A program takes back its memory through the same capability,** wherever the memory went ([0064](0064-memory-belongs-to-payer.md)).
- **03's and 05's questions on memory requests and payers are answered,** together with [0138](0138-pager-keeps-reservations.md).
