# 0068. The kernel has no heap

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Programs pay for kernel objects ([0026](0026-programs-pay-kernel-objects.md)), user mode builds page tables ([0052](0052-user-mode-page-tables.md)), the kernel has one stack per core ([0060](0060-kernel-stack-per-core.md)), and the object window's page tables are set aside at boot ([0067](0067-object-window-offset.md)). The roadmap still planned a kernel heap, and `alloc` in the kernel, for weeks 3–5.

## Options

1. **No heap.**
2. **A small fixed heap,** sized at boot for the kernel's internal needs.

## Decision

Option 1.

- The kernel has no heap.
- All kernel memory is either set aside at boot or made of kernel objects that programs pay for. What's set aside is the kernel's code and data, its per-core stacks, and the object window's page tables.
- The kernel builds on `core` only and never uses `alloc`.

## Consequences

- **The kernel can't run out of memory at run time.**
- **Every kernel data structure is either sized at boot or lives in a kernel object.**
- **There's no allocator to check,** which keeps the kernel smaller and easier to prove later ([0018](0018-kernel-assurance.md), [0019](0019-kernel-minimality.md)).
- **The roadmap changes.** "Kernel heap" and "`alloc` in the kernel" are dropped from weeks 3–5 ([README](../../README.md)).
