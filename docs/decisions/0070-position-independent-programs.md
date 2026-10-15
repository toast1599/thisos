# 0070. Programs are built position-independent

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0061](0061-no-randomization-yet.md) keeps randomization possible. Packages are immutable and named by their content hash ([0041](0041-immutable-packages.md)), so a program can't be relinked after it ships.

## Options

1. **Position-independent.**
2. **Fixed addresses.**

## Decision

Option 1. Every program is built position-independent.

## Consequences

- **Randomization can be turned on later in the loader** ([0031](0031-user-mode-loader.md)), without rebuilding any package.
- **The cost is close to nothing** on x86_64 and AArch64, which reach code and data relative to the instruction pointer.
- **The loader relocates each program as it places it.** Which relocations a program may carry belongs to the program format ([02-verification](../design/02-verification.md)).
