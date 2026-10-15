# 0063. The kernel turns on the widest virtual addressing the CPU offers

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

x86_64 CPUs offer 48-bit virtual addresses, and some offer 57 bits with 5-level paging. AArch64 CPUs offer 48 bits, and some offer 52. Wider addresses can add a page-table level, which costs a little on each TLB miss. Under [0012](0012-no-memory-cap.md), programs are limited only by the hardware.

## Options

1. **Widest available.**
2. **48 bits unless set.** Wider addresses only when the owner's settings ask for them.

## Decision

Option 1.

- The kernel turns on the widest virtual addressing the CPU offers.
- Programs are limited only by the hardware ([0012](0012-no-memory-cap.md)).

## Consequences

- **The number of page-table levels is known only at boot.** The page-table code in user mode ([0052](0052-user-mode-page-tables.md)) handles every depth the architecture allows.
- **Programs get the lower half** ([0066](0066-kernel-upper-half.md)), whose size follows from this.
- **TLB misses cost a little more** on CPUs where the extra level is turned on.
- **Detecting what the CPU offers is architecture-specific** ([0008](0008-architecture-neutral-design.md)).
- **Programs that keep tags in the upper bits of pointers,** as some JavaScript engines do, need addresses that stay low. How they ask for that is open ([03-memory](../design/03-memory.md)).
