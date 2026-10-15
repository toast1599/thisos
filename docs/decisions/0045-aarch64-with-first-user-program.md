# 0045. AArch64 is brought up together with the first user-mode program

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0011](0011-aarch64-second.md) made AArch64 the second architecture. When it has to be up was left open; the only guidance was "later, but not too late" ([0009](0009-x86-64-first.md)).

## Options

1. **At the first user-mode program,** around weeks 7 to 9 of the roadmap.
2. **After the base system,** around month 3.
3. **From the first boot.**

## Decision

Option 1. AArch64 boots and runs the first user-mode program in the same phase as x86_64, around weeks 7 to 9 of the roadmap.

## Consequences

- **x86_64-only assumptions are caught early,** while the kernel is still small. Strong memory ordering is the main one ([0009](0009-x86-64-first.md)).
- **QEMU for AArch64 must be installed by then** ([0011](0011-aarch64-second.md)).
- **CI covers both architectures from then on** ([07-engineering](../design/07-engineering.md)).
