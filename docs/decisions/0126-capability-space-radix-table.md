# 0126. A capability space is a radix table, and capabilities are named by index

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

A program can use only the objects it holds capabilities to ([0020](0020-capabilities.md)). Its capabilities are kept in kernel memory it pays for ([0026](0026-programs-pay-kernel-objects.md)), and the kernel records what was made from what so that revoke can find it ([0053](0053-revoke-destroys-derived.md)). seL4 stores capabilities in CNodes: tables of any power-of-two size, linked into a tree with guard bits, where a capability is named by a bit path through the tree.

## Options

1. **A radix table.** Fixed-size tables linked like a page table, with capabilities named by a plain index.
2. **seL4's CNode tree.**
3. **One flat table** per program, its size fixed when it's made.

## Decision

Option 1.

- A capability space is a tree of fixed-size tables of capability slots, linked like a page table.
- The tables are kernel objects made from the program's memory ([0026](0026-programs-pay-kernel-objects.md)).
- A capability is named by a plain 64-bit index into its program's capability space.
- A program grows its capability space by adding tables.
- The tree's depth and the tables' size are chosen during implementation ([0073](0073-kernel-layout-in-implementation.md)). Names don't depend on them.

## Consequences

- **Lookup is a short, fixed walk,** with no guard bits to check.
- **Names are plain integers** to std and the bindings, which wrap them in typed handles ([0142](0142-own-idl.md)).
- **A program pays for its capability slots,** including the space each slot's revoke records take ([0053](0053-revoke-destroys-derived.md)). That's why memory figures cover capabilities ([0111](0111-resource-accounting.md)).
- **The kernel never allocates a slot itself.** A capability that arrives in a message lands in a slot the receiver already has ([0123](0123-capabilities-in-messages.md)).
- **05's question on the capability design is answered,** together with [0127](0127-capability-space-per-program.md) to [0131](0131-namespace-path-prefixes.md).
