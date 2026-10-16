# 0077. Swap is a feature of the default pager; it comes later and is encrypted

- **Status:** Accepted, amended by [0082](0082-corrections.md)
- **Date:** 2026-10-15

## Context

Under [0048](0048-reserve-on-grant.md), swap raises how much memory can be reserved. Under [0049](0049-memory-fair-share.md), pagers page out memory when real memory is short. Who writes memory to storage, and when that arrives, was open.

## Options

1. **The default pager, later.**
2. **Compressed memory first,** before any swapping to storage.
3. **No swap.**

## Decision

Option 1.

- Swap is a feature of the default pager. It pages out memory for the programs it serves, through the storage service.
- Swap comes once the filesystem and storage exist. Until then, only asking and stopping apply ([0049](0049-memory-fair-share.md)).
- Pages written out are encrypted with a key that is made at boot and never leaves memory.
- Programs with their own pagers may page out their own memory.

## Consequences

- **Storage never holds program data in the clear,** and swapped pages can't be read after shutdown, because the key is gone.
- **Once swap exists, more memory can be reserved** ([0048](0048-reserve-on-grant.md)).
- **Where swapped pages are stored, and how the default pager reaches the storage service,** come with the storage design ([06-system](../design/06-system.md)).
- **03's paging-out question is answered.**
