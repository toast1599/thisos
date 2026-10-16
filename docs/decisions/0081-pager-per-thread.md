# 0081. Page faults go to the faulting thread's pager

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

[0027](0027-user-mode-pagers.md) forwards each page fault to the faulting program's pager, and [0056](0056-pager-chosen-at-spawn.md) has whoever spawns a program name its pager. 0056 also lets a program have a pager thread of its own, whose own faults go to its pager one level up ([0057](0057-pager-fault-chain.md)). If faults followed the program, that thread's faults would go to the thread itself, which 0027 forbids.

A consistency review on 2026-10-16 found this conflict. [05-interfaces](../design/05-interfaces.md) already speaks of a thread's pager.

## Options

1. **Per thread.** Each thread names its pager when it's created, as in seL4.
2. **Per region.** The kernel sends each fault to the pager of the region the faulting address is in, as Mach and Fuchsia do.
3. **Per program,** with no pager thread inside the program it pages.

## Decision

Option 1.

- Each thread names its pager when it's created. The kernel sends that thread's page faults to it.
- A program's threads normally share the pager its spawner chose ([0056](0056-pager-chosen-at-spawn.md)).
- A pager thread inside a program names a pager one level up ([0057](0057-pager-fault-chain.md)).
- Pagers that share an address space split its regions between them in user mode.
- This amends 0027, whose faults now go to the faulting thread's pager rather than the program's, and 0056, whose spawner names the pager that the program's threads share.

## Consequences

- **The kernel's part stays small.** It keeps one pager per thread, as seL4 does, and nothing about regions ([0019](0019-kernel-minimality.md)).
- **Pager threads inside a program work as 0056 intends.** The pager thread's faults go one level up, and the other threads' faults go to it.
- **Pagers that share an address space have to agree** on which of them maps which region.
- **Whether other faults go to the same handler** is open ([04-execution](../design/04-execution.md)).
- **Whether a thread's pager can be changed after it's created, and by whom,** is part of the capability design ([05-interfaces](../design/05-interfaces.md)).
