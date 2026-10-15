# 0054. The kernel enforces the right to make memory executable

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0043](0043-executable-memory-capability.md) requires a capability to make memory executable, and forbids memory that is writable and executable at once. It left open whether the kernel or the pager ([0027](0027-user-mode-pagers.md)) enforces this, and whether "at once" applies to one mapping or to the memory itself. A program can have a pager thread of its own ([0056](0056-pager-chosen-at-spawn.md)).

## Options

1. **In the kernel.** Execute permission is a right carried on a memory capability.
2. **In the pager.** Pagers refuse executable mappings unless the program holds the capability.

For what "never writable and executable at once" applies to:

- **Each piece of memory,** across every mapping in every address space.
- **Each mapping,** so the same memory may be mapped writable at one address and executable at another.

## Decision

Option 1, applied to each piece of memory.

- Execute permission is a right carried on a memory capability.
- No memory is mapped executable anywhere while it's mapped writable anywhere, in any address space. The kernel refuses any mapping that would break this.
- The kernel refuses any executable mapping made through a capability without the execute right.
- Switching memory between writable and executable goes through the kernel, which does the cache and TLB upkeep this needs.

## Consequences

- **No pager can get around [0043](0043-executable-memory-capability.md),** not even a program acting as its own pager.
- **No program can rewrite running code through a second mapping.** JIT engines switch memory between writable and executable through the kernel, instead of mapping it both ways at two addresses.
- **The strict rule can be relaxed later** without breaking programs, while tightening a looser rule later would break them.
- **The kernel checks every mapping of a piece of memory.** It already records every mapping, so revoke can remove them ([0053](0053-revoke-destroys-derived.md)).
- **The loader holds the execute right** to map code from signed packages ([0031](0031-user-mode-loader.md), [0040](0040-signed-programs-owner-policy.md)). JIT engines hold it under 0043.
- **The upkeep is architecture-specific** ([0008](0008-architecture-neutral-design.md)). AArch64 needs instruction-cache upkeep. Both architectures need TLB flushes where the memory was mapped writable.
- **03's question on making memory executable is answered.**
