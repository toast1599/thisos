# 0037. The kernel is built with one pinned, audited toolchain

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Under [0014](0014-microkernel.md), the compiler no longer isolates programs, but it still builds the kernel, where a miscompilation is a kernel bug. Day 2 found 77 open rustc soundness holes that apply to this OS, and hundreds of open miscompilation reports in LLVM ([02-verification](../design/02-verification.md)).

## Options

1. **A pinned, audited toolchain.**
2. **Option 1 plus review** of the generated machine code for the most critical paths.
3. **Option 1 plus a check** of the compiled kernel against its source, as seL4 does.

## Decision

Option 1.

- The kernel is built with one pinned toolchain version.
- Each upgrade is deliberate and reviewed.
- Correctness comes from the tests and fuzzing required by [0018](0018-kernel-assurance.md).

## Consequences

- **Upgrades are reviewed like kernel changes.** How often they happen, and whether the toolchain is upstream nightly or a fork, is open ([07-engineering](../design/07-engineering.md)).
- **Stronger checks stay possible.** Machine-code review and checking the binary can still be added later, alongside the proofs that 0018 leaves for later.
