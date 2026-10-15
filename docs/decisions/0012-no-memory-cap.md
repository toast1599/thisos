# 0012. No fixed limit on program memory

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

The cheapest software isolation mechanisms work by giving every program a region of fixed size and forcing every address into it. WebAssembly with 32-bit memory and LFI as published both use 4 GiB regions. Measured on 2026-10-15, removing the fixed region makes isolation by explicit bounds checks cost +41% to +62% over native code ([02-verification](../design/02-verification.md)).

## Options

1. A fixed limit per program, set by the design (for example 4 GiB), in exchange for cheaper isolation.
2. No fixed limit.

## Decision

Option 2. Nothing in the design imposes a fixed limit on how much memory a program can use. A program is limited only by the hardware and by the memory actually available.

## Consequences

- Isolation mechanisms that depend on a fixed-size region per program are ruled out. That includes 32-bit WebAssembly and LFI as published ([02-verification](../design/02-verification.md)).
- How many address bits are usable differs by architecture ([0010](0010-64-bit-only.md)). That is a hardware limit, not a design limit.
- Still open: whether limits set by policy, such as quotas from resource accounting, count as artificial limits ([04-execution](../design/04-execution.md)).
