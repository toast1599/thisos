# 0048. Memory is reserved when it's granted; the system never promises more than it has

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Under [0025](0025-user-mode-memory.md), a user-mode memory service hands out memory. Linux overcommits: grants always succeed, and real memory is found the first time a page is touched, so running out happens as a fault in the middle of a program's work. Windows charges each grant against memory and page file when it's made.

Thread stacks need a rule of their own, because they grow by faulting.

## Options

1. **Reserve on grant.** A grant is backed by real memory, or later by swap, from the moment it's made.
2. **Overcommit.** Grants always succeed, and memory is found on first touch.

For thread stacks:

- **Reserve as they grow,** as on Windows.
- **Reserve in full** when the thread is created.

## Decision

Option 1, with stacks that reserve as they grow.

- Memory granted to a program is reserved for it at once. It's backed by real memory or, once swap exists, by swap space.
- A request that can't be met fails when it's made, with an error the program can handle.
- Reserving address space without memory behind it is free, and doesn't count against what's available.
- A thread stack reserves its address range up front and reserves memory as it grows. If memory runs out while a stack grows, the thread gets a stack-overflow fault. This is the one case where a fault can fail for lack of memory.

## Consequences

- **Running out is reported where it can be handled:** at the request, not at a fault mid-work. What std does when an allocation fails belongs to the std mapping ([06-system](../design/06-system.md)).
- **Apart from stack growth, a fault on granted memory never fails for lack of memory.** A pager can always back what was granted ([0027](0027-user-mode-pagers.md)).
- **Stack growth fails only during a shortage,** when [0049](0049-memory-fair-share.md) doesn't serve the request, for example for a program over its fair share.
- **Threads hold only the stack they use.** A stack reserves address space for its full size, but memory only as it grows.
- **Sparse use costs only what it reserves.** Heaps and arenas reserve address space freely and reserve memory as they grow.
- **Memory is used less fully than with overcommit.** Memory that's reserved but untouched can't go to anyone else. Swap, once it exists, raises how much can be reserved.
- **Contention is about reservations.** When reservations would exceed what's available, [0049](0049-memory-fair-share.md) decides.
