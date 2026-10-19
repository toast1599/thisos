# 0271. Corrections to 0009, 0045, 0206 and 0227

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The audit after the freeze found lines in Accepted records that are wrong as written ([0256](0256-audit-findings-as-evidence.md)). Records are never edited after the fact, so a later record corrects them, as [0082](0082-corrections.md), [0221](0221-corrections.md) and [0254](0254-corrections.md) did.

## Options

1. **Amend, with a status note,** as in 0082, 0221 and 0254.
2. **Leave them as history.**

## Decision

Option 1. Each corrected record keeps its decision, and its status gains "amended by 0271". These lines are corrected:

- **0009, Consequences.** "The second architecture is what finds them": QEMU's emulation of AArch64 on an x86_64 host doesn't show weaker ordering ([0256](0256-audit-findings-as-evidence.md)). Host tests on arm64, loom and Miri find such bugs ([0263](0263-arm64-host-tests.md)).
- **0045, Consequences.** "x86_64-only assumptions are caught early... Strong memory ordering is the main one": bringing AArch64 up early still catches assumptions such as page sizes and the interrupt controller. Ordering is caught by host tests on arm64, loom and Miri ([0263](0263-arm64-host-tests.md)).
- **0206, Decision.** "Packages signed by the system image's signer always run": the signer is the root key behind the running image. Packages signed by any working key that its statements name run without the owner's approval, the project's packages outside the image included ([0205](0205-root-and-working-keys.md)). On an image an owner builds with a root key of their own, the same holds for that root's keys.
- **0227, Consequences.** "AArch64's weaker memory ordering under load shows up early": QEMU's tests on AArch64 don't show it. Host tests on arm64, loom and Miri check ordering ([0263](0263-arm64-host-tests.md)). Placing multicore before drivers still tests concurrency early, on x86_64 and in the simulator.

## Consequences

- **The four records stay Accepted,** and their status links here.
- **None of the corrections is a decision of its own.** Each restates what 0205 or 0263 decides.
