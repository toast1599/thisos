# 0138. Pagers keep a program's address-space reservations

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Reserving address space without memory behind it is free, while memory granted to a program is reserved at once ([0048](0048-reserve-on-grant.md)). Programs that need low addresses say so when they reserve address space ([0078](0078-low-addresses-on-request.md)). The kernel knows only page tables ([0052](0052-user-mode-page-tables.md)) and keeps nothing about regions, and pagers that share an address space split its regions between them in user mode ([0081](0081-pager-per-thread.md)).

## Options

1. **In the pager.** A program reserves and backs address space by calling its pager.
2. **Regions in the kernel,** as Zircon's VMARs are.

## Decision

Option 1.

- A program reserves address space by calling its pager, with a size and, if it needs low addresses, the highest address it can use ([0078](0078-low-addresses-on-request.md)). Reserving is free ([0048](0048-reserve-on-grant.md)).
- Asking the pager to back a region gets memory for it from the memory service at once ([0137](0137-memory-capability-per-program.md)). A request that can't be met fails right there, with an error the program can handle ([0048](0048-reserve-on-grant.md)).
- The pager maps the memory in on faults, and makes page tables from the program's memory ([0052](0052-user-mode-page-tables.md)).
- The loader works the same way while it builds a new program ([0031](0031-user-mode-loader.md)).

## Consequences

- **The kernel stays out of regions** ([0019](0019-kernel-minimality.md), [0081](0081-pager-per-thread.md)).
- **Stacks are regions that reserve memory as they grow** ([0048](0048-reserve-on-grant.md)), through the same calls.
- **std's allocator calls the pager** to reserve and back its heap, through the bindings ([0145](0145-std-over-native-api.md)).
- **0078's question on asking for low addresses is answered, and so is 0052's on how the loader and pagers reach a program's memory.**
