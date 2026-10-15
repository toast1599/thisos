# 0052. Page tables are kernel objects that user mode builds

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Under [0025](0025-user-mode-memory.md) and [0026](0026-programs-pay-kernel-objects.md), programs pay for kernel objects with memory they hold, and the kernel keeps no allocator for them. Page tables are one of the larger uses of kernel memory. In seL4, user mode makes page tables from memory it holds and links them into an address space, and the kernel checks each step.

## Options

1. **User mode builds them** from page-table objects.
2. **The kernel builds them** behind one architecture-neutral address-space object, using memory the program set aside.

For which program builds them, once the memory service, the default pager and the loader are separate programs ([0071](0071-three-memory-programs.md)):

- **The loader and the pager,** which already build and change address spaces.
- **The memory service,** asked by the loader and the pager.

## Decision

Option 1, built by the loader and the pager.

- Page tables are kernel objects. User mode makes them and links them into an address space, and the kernel checks every step, so a page table can only map memory its builder holds a capability for.
- The loader makes a new program's page tables when it builds its address space ([0031](0031-user-mode-loader.md)). The program's pager makes and changes them when it maps memory on faults ([0027](0027-user-mode-pagers.md)).
- Both make page tables from the program's own memory, so programs pay for their page tables ([0026](0026-programs-pay-kernel-objects.md)).
- At boot, the kernel builds the root task's first address space itself, because nothing in user mode exists yet to do it. This is the one exception.

## Consequences

- **The kernel needs no page-table allocator** ([0019](0019-kernel-minimality.md), [0026](0026-programs-pay-kernel-objects.md)).
- **A fault that needs a new page table takes no extra hop.** The pager makes the table itself.
- **Page-table shape is visible to user mode.** Levels and page sizes differ by architecture ([0069](0069-all-page-sizes.md)), and the number of levels is known only at boot ([0063](0063-widest-addressing.md)). The code that builds page tables is architecture-specific. It lives in an architecture-specific part of a library shared by the loader and pagers ([0008](0008-architecture-neutral-design.md), [0046](0046-scope-of-0008.md)), and ordinary programs never see it.
- **The kernel half is filled in by the kernel** ([0066](0066-kernel-upper-half.md)). Where both halves share one top-level table, as on x86_64, the kernel copies its own entries into each new top-level page table, as seL4 does. On AArch64, the kernel half has a table of its own.
- **Destroying a page table unmaps everything under it,** through revoke ([0053](0053-revoke-destroys-derived.md)).
- **How the kernel maps the root task without parsing an executable** ([0031](0031-user-mode-loader.md)) is designed with the boot path ([06-system](../design/06-system.md)).
- **How the loader and pagers reach a program's memory** is designed with the interfaces ([05-interfaces](../design/05-interfaces.md)).
