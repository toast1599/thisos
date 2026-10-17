# 0170. Memory mapped for a device is held back after a revoke, until the IOMMU service releases it

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The IOMMU service keeps the devices' I/O page tables in its own memory ([0169](0169-iommu-service.md)), so the kernel can't see them. Revoking memory destroys everything made from it and removes every mapping of it, without anyone's cooperation ([0053](0053-revoke-destroys-derived.md)). The kernel clears memory before it's made into anything new ([0051](0051-kernel-clears-memory.md)). Without a rule, revoked memory could be cleared and handed to another program while a device can still write to it.

## Options

1. **Quarantine until released,** as Zircon does with pinned pages.
2. **Revoke waits for the service** to remove the device mapping.
3. **Move the IOMMU into the kernel.**

## Decision

Option 1.

- When the IOMMU service maps memory for a device, it marks that memory in the kernel, through the memory's capability and the DMA capability ([0169](0169-iommu-service.md)).
- Revoking marked memory still ends every program's access at once ([0053](0053-revoke-destroys-derived.md)).
- The kernel then holds the memory, neither cleared nor reusable, until the IOMMU service reports that the device mapping is gone and the IOMMU has been flushed.
- This amends 0053, whose revoke no longer removes device mappings itself.

## Consequences

- **No device can write into memory that's been reused.** Memory is reused only after its device mappings are gone.
- **Revoke never waits** ([0053](0053-revoke-destroys-derived.md), [0065](0065-lost-memory-is-an-error.md)). Only reuse waits, and only on the trusted IOMMU service.
- **Memory can come back late.** The memory service ([0049](0049-memory-fair-share.md)) may have to wait for held-back memory before it can grant it again.
- **The service's own tables are marked too,** since the IOMMU reads them. The IOMMU never reads memory that has been reused.
- **Only the DMA capability marks memory.** A program that could mark memory itself could keep it from ever being reused, so marking and releasing need that capability ([0016](0016-threat-model-scope.md)).
- **The kernel gains a mark on memory and a call to release it,** but no IOMMU code.
- **06's question on DMA and revoke is answered.**
