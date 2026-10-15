# 0069. The kernel supports every page size the hardware offers

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Page tables are built in user mode ([0052](0052-user-mode-page-tables.md)), and nothing outside architecture-specific code may assume a page size ([0008](0008-architecture-neutral-design.md)). x86_64 offers 4 KiB, 2 MiB and 1 GiB pages. AArch64 offers 4, 16 or 64 KiB granules, each with its own larger page sizes, and the granule is chosen at boot.

## Options

1. **Every size the hardware offers.**
2. **4 KiB only at first,** with larger pages later.

## Decision

Option 1.

- The kernel accepts every page size the hardware offers.
- The loader and pagers choose which sizes to use ([0052](0052-user-mode-page-tables.md)). Where asked, the memory service hands out memory in pieces sized and aligned for large pages.
- AArch64 starts with the 4 KiB granule, the most widely supported. The 16 and 64 KiB granules can follow.

## Consequences

- **Programs that use a lot of memory pay less in TLB misses.**
- **Nothing outside architecture-specific code assumes a page size** ([0008](0008-architecture-neutral-design.md)).
- **Once other granules follow, the granule is known only at boot,** like the number of page-table levels ([0063](0063-widest-addressing.md)), so user-mode page-table code has to handle it ([0052](0052-user-mode-page-tables.md)).
- **03's page-size question is answered.**
