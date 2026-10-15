# 0018. Kernel assurance: review, tests and fuzzing from the start; formal proofs later

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0014](0014-microkernel.md) requires the kernel to be heavily audited and tested. The options range from testing alone to a machine-checked proof of the whole kernel, as seL4 has.

## Options

1. Strict review, high test coverage and fuzzing from the start; formal proofs later.
2. The same, plus proofs of the most critical parts, such as IPC and page-table management, from the start.
3. A machine-checked proof of the whole kernel.

## Decision

Option 1.

- The kernel has strict review rules, high test coverage, and fuzzing of the system-call interface from the start.
- Its code is kept simple enough that parts of it can be proven formally later.

## Consequences

- **The details come on Day 7:** review rules, coverage targets and the fuzzing setup ([07-engineering](../design/07-engineering.md)).
- **Keeping proofs possible shapes the code.** The kernel stays small and simple, with clear invariants, which reinforces [0019](0019-kernel-minimality.md).
- **Testing and review are the only assurance at first.** No correctness proof exists yet.
