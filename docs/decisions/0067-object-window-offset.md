# 0067. Kernel objects are mapped at their physical address plus a fixed offset

- **Status:** Accepted, amended by [0096](0096-object-window-lazy-unmap.md)
- **Date:** 2026-10-15

## Context

Under [0055](0055-kernel-maps-only-objects.md), the kernel maps kernel objects but no program data, so kernel objects need a place in the kernel's half ([0066](0066-kernel-upper-half.md)). [0019](0019-kernel-minimality.md) and [0026](0026-programs-pay-kernel-objects.md) keep allocators out of the kernel.

## Options

1. **Physical address plus a fixed offset,** with the window's page tables set aside at boot.
2. **Placed on demand** at free addresses, with page tables made from the memory of the program that made each object.

## Decision

Option 1.

- A kernel object is mapped at its physical address plus a fixed offset, and only while it's a kernel object.
- The page tables for this window are set aside at boot.

## Consequences

- **Mapping an object never needs memory from anyone,** and the kernel needs no address allocator.
- **Finding an object from its physical address takes one addition.**
- **The set-aside is about 0.2% of memory:** one 8-byte entry for every 4 KiB page. It's fixed overhead, like the kernel's code, not a pool for objects ([0026](0026-programs-pay-kernel-objects.md)).
- **Destroying an object unmaps it,** which needs a TLB flush on every core ([0055](0055-kernel-maps-only-objects.md)).
- **The window spans the physical address range,** inside the kernel's half.
