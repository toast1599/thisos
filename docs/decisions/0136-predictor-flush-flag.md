# 0136. A program asks for predictor flushes with a flag on its address space

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The kernel flushes the branch predictors when it switches to or away from a program that asks ([0080](0080-predictor-flush-both-ways.md)). Switching between programs means switching between address spaces ([0083](0083-program-is-address-space.md)). A program that holds secrets, such as a key-management service, needs the protection from its first instruction.

## Options

1. **A flag on the address space.**
2. **A flag on each thread.**

## Decision

Option 1.

- Each address space has a flag that asks for flushes, set through its capability with the control right ([0128](0128-rights-and-badges.md)).
- The loader sets it before the program's first instruction when the program's image declares it ([0147](0147-elf-declarations-only.md)).
- The program can set or clear it later.

## Consequences

- **A program can be covered from its first instruction.**
- **Only the program, and whoever builds it, decide.** The flush protects the program itself, so nobody else needs a say.
- **Flags on threads would leave a gap.** A flagged thread entered from an unflagged thread of the same program would run on whatever the predictors learned before.
- **0080's question on how a program asks is answered.**
