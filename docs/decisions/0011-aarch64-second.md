# 0011. AArch64 is the second architecture

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

x86_64 is the first architecture ([0009](0009-x86-64-first.md)). A second architecture comes "later, but not too late". Until it exists, the architecture-neutral design ([0008](0008-architecture-neutral-design.md)) can't be tested by a real port. The candidates were AArch64 and RISC-V.

## Options

1. AArch64
2. RISC-V

## Decision

Option 1. AArch64 is the second architecture. The point by which it has to be up is still open ([01-objective](../design/01-objective.md)).

## Consequences

- AArch64 is where the assumptions that x86_64 lets slide get tested ([0009](0009-x86-64-first.md)):
  - memory ordering is weaker than on x86_64
  - pages can be 4, 16 or 64 KiB
- Kernel mode is EL1. EL2 and EL3 run above it ([0008](0008-architecture-neutral-design.md)).
- The calling convention on AArch64 is AAPCS64 ([0007](0007-abi-calls-and-interfaces.md)).
- QEMU for AArch64 has to be installed on the development machine as part of the bring-up ([0009](0009-x86-64-first.md)).
