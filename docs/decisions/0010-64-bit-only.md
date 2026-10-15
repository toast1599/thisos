# 0010. 64-bit architectures only

- **Status:** Accepted
- **Date:** 2026-10-14

## Context

Pointer and length sizes have to be fixed in the ABI ([0007](0007-abi-calls-and-interfaces.md)), and generic code needs to know what it may assume about word size ([0008](0008-architecture-neutral-design.md)).

## Options

1. 64-bit and 32-bit architectures.
2. 64-bit architectures only.

## Decision

Option 2. Only 64-bit architectures are supported, with no regard for 32-bit at all. There are no 32-bit targets, no 32-bit compatibility modes, and no 32-bit programs.

## Consequences

- Every supported architecture guarantees two things:
  - Pointers and `usize` are 64 bits wide, so the ABI fixes pointer and length fields at 64 bits.
  - 64-bit atomic operations are available.
- Because these hold on every supported architecture, generic code may rely on them. They don't count as architecture-specific concepts under [0008](0008-architecture-neutral-design.md).
- How many pointer bits are actually usable for addresses does differ: 48 or 57 on x86_64, 48 or 52 on AArch64, 39, 48 or 57 on RISC-V. So that stays architecture-specific. Generic code can't assume an address width, and it can't store tags in the upper bits of a pointer.
