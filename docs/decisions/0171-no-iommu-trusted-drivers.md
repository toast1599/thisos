# 0171. Without an IOMMU, the drivers of DMA-capable devices are trusted

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The IOMMU service confines the devices' DMA ([0169](0169-iommu-service.md)). Some machines have no IOMMU, such as some small AArch64 boards.

## Options

1. **The system runs, and those drivers are trusted.**
2. **An IOMMU is required,** and machines without one aren't supported.

## Decision

Option 1.

- On a machine without an IOMMU, the system runs, and the drivers of DMA-capable devices count as trusted base.
- The hardware-discovery service reports that there's no IOMMU.
- A boot setting can require an IOMMU ([0157](0157-boot-settings-in-ramdisk.md)). With it set, those drivers aren't started on a machine without one.
- Without an IOMMU, the root task gives those drivers the DMA capability ([0169](0169-iommu-service.md)). With it they learn the physical addresses of their DMA memory, and mark it as the IOMMU service would ([0170](0170-dma-quarantine.md)).

## Consequences

- **The trusted base grows on such machines** by the DMA-capable drivers. They're OS services, bound by [0017](0017-unsafe-code-kernel-and-services.md) ([0044](0044-system-image-is-os-shipped.md)).
- **Owners can refuse that** with the boot setting.
- **Physical addresses stay hidden from everyone else.** Only these drivers and the IOMMU service ([0169](0169-iommu-service.md)) can learn them.
