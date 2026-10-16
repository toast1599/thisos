# 0099. The kernel doesn't pause a sibling hardware thread when it runs

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

[0034](0034-smt-trusted-pairs.md) pairs programs on a core, but the kernel runs on both hardware threads of a core, next to whatever program the sibling runs. Speculative leaks from programs into the kernel are in scope ([0016](0016-threat-model-scope.md)).

## Options

1. **CPU mitigations only.** The kernel relies on mapping little, and on the CPU's documented mitigations.
2. **Pause the sibling.** When one hardware thread enters the kernel, its sibling enters too and waits until both can leave.

## Decision

Option 1.

- The kernel doesn't pause a sibling hardware thread when it enters the kernel.
- It relies on mapping only its own memory and kernel objects ([0055](0055-kernel-maps-only-objects.md)), and on the CPU's documented mitigations for flaws that leak between siblings ([0100](0100-cpu-flaw-mitigations.md)).

## Consequences

- **System calls cost nothing extra on the sibling.**
- **What the kernel maps stays within a sibling's reach** on CPUs whose flaws can't be fully mitigated, such as the saved registers in thread objects. That's within the best-effort defence of [0016](0016-threat-model-scope.md).
