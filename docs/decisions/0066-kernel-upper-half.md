# 0066. The kernel sits in the upper half of every address space, and programs in the lower half

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Every address space holds the kernel ([0047](0047-kernel-mapping.md)) and one program ([0014](0014-microkernel.md)). Both architectures split virtual addresses into a lower and an upper half, and AArch64 uses a separate translation table for each half.

## Options

1. **Kernel in the upper half.**
2. **Kernel in the lower half.**

## Decision

Option 1. The kernel sits in the upper half of every address space, and each program gets the lower half.

## Consequences

- **It follows both architectures' conventions,** as Linux and seL4 do.
- **A program's usable range is the lower half,** whose size depends on the addressing the CPU offers ([0063](0063-widest-addressing.md)).
- **On AArch64 the kernel half has its own table,** so nothing is copied into programs' page tables. On x86_64 the kernel copies its entries into each new top-level page table ([0052](0052-user-mode-page-tables.md)).
- **The rest of the layout is open.** That covers where the kernel's code, data and stacks sit in the upper half, and how the loader lays out a program's half ([03-memory](../design/03-memory.md)).
