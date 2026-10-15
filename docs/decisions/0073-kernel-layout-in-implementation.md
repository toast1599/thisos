# 0073. The kernel's internal layout is chosen during implementation

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Two things were still open: where the kernel's code, data and per-core stacks sit in the upper half ([0066](0066-kernel-upper-half.md)), and where the kernel sits in physical memory. No interface shows kernel addresses to programs ([0061](0061-no-randomization-yet.md)).

## Options

1. **Left to implementation,** within the rules already set.
2. **Fixed in the design** now, for each architecture.

## Decision

Option 1.

- The layout of the kernel's code, data and per-core stacks in the upper half, and the kernel's place in physical memory, are chosen during implementation.
- They stay within the rules already set: the upper half ([0066](0066-kernel-upper-half.md)), the object window ([0067](0067-object-window-offset.md)), and guard pages around stacks ([0060](0060-kernel-stack-per-core.md)).

## Consequences

- **The layout can change without breaking anything,** because nothing outside the kernel depends on it ([0061](0061-no-randomization-yet.md)).
- **It's not part of the frozen design,** so changing it needs no new record.
- **Architecture constraints are met in the implementation.** For example, x86_64's kernel code model needs the kernel image in the top 2 GiB of addresses.
