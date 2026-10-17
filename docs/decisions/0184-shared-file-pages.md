# 0184. Read-only file pages are shared from the service's cache, which pays for them

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Page faults go to the faulting thread's pager ([0081](0081-pager-per-thread.md)), normally the default pager ([0056](0056-pager-chosen-at-spawn.md)). Clients pay for the state services keep for them, and state shared by all clients, such as a cache, stays in the service's own memory ([0050](0050-clients-pay.md)). Memory belongs to its payer, which can always take it back ([0064](0064-memory-belongs-to-payer.md)). Apart from stack growth, a fault on granted memory never fails for lack of memory ([0048](0048-reserve-on-grant.md)). Installed programs are immutable packages ([0041](0041-immutable-packages.md)).

## Options

1. **Shared from the service's cache.**
2. **Copies for everyone,** in each program's own memory.

For keeping 0048's promise once pages are shared:

- **Keep reservations while pages are mapped.**
- **A second failure case,** amending 0048.
- **Programs reserve for their mappings,** in their own accounts.

## Decision

Option 1, with reservations kept while pages are mapped.

- The default pager maps files and package contents into programs.
- For read-only mappings, such as code, it maps pages from the package store's or the filesystem's cache. The service pays for those pages, as state shared by all its clients.
- Writable private mappings get copies in the program's own memory.
- The service keeps a reservation for every cached page that's mapped anywhere. When real memory is short, it can drop a mapped page's contents: the page's mappings are removed, and the next touch reads it back into that reservation, as paging out does ([0049](0049-memory-fair-share.md)).
- The service gives back reservations only for pages that nothing maps.
- Writable mappings that write back to the file come later.

## Consequences

- **Programs running the same package share its code pages.**
- **No program can take shared pages away from the others,** since none of them pays for them ([0064](0064-memory-belongs-to-payer.md)).
- **The cache counts against the service's share** like any other memory ([0049](0049-memory-fair-share.md)). Asked to give memory back, the service gives back pages that nothing maps.
- **A fault on a file page never fails for lack of memory,** as 0048 requires, since a mapped page's reservation stays with the service. It's an ordinary fault, not memory taken back ([0065](0065-lost-memory-is-an-error.md)), because the pager knows the region is backed by a file.
- **Code pages carry the execute right.** The package store gives it only for code from signed packages, to the loader and the default pager, which map that code ([0040](0040-signed-programs-owner-policy.md), [0054](0054-kernel-enforces-execute.md)).
- **06's question on file-backed memory is answered.**
