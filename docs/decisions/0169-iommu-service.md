# 0169. A user-mode IOMMU service keeps the devices' I/O page tables

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Without an IOMMU, a device could write anywhere in memory, and 0014 listed DMA confined by an IOMMU among what drivers need from the kernel ([0014](0014-microkernel.md)). When memory is revoked, a device's mappings of it have to go before the memory is cleared and reused ([0051](0051-kernel-clears-memory.md), [0053](0053-revoke-destroys-derived.md)), and taking memory back never waits on its users ([0065](0065-lost-memory-is-an-error.md)). Under [0032](0032-minimal-hardware-discovery.md), user mode finds the hardware.

## Options

1. **The kernel programs the IOMMU,** told by user mode where it is, with I/O page tables as kernel objects built by user mode.
2. **A user-mode IOMMU service.**
3. **The kernel finds IOMMUs itself** in the firmware tables, amending 0032.

## Decision

Option 2.

- A user-mode IOMMU service programs the IOMMU. It holds the IOMMU's registers, and keeps each device's I/O page tables in its own memory.
- A driver gets DMA access to memory by passing the service a capability for it. The service maps only memory it holds a capability for.
- The service holds the DMA capability, a kernel capability that the root task receives at boot ([0159](0159-root-task-boot-information.md)). With it, the service learns the physical address of memory it holds a capability for, and marks memory for devices ([0170](0170-dma-quarantine.md)).
- The hardware-discovery service finds the IOMMUs, and the root task hands their registers to the IOMMU service ([0032](0032-minimal-hardware-discovery.md), [0168](0168-device-memory-and-ports.md)).
- This amends 0014, whose drivers no longer need DMA confined by the kernel.

## Consequences

- **The kernel gains no IOMMU code** ([0019](0019-kernel-minimality.md)).
- **The IOMMU service joins the trusted base.** It controls every device's DMA. As an OS service, it's bound by [0017](0017-unsafe-code-kernel-and-services.md).
- **The kernel can't see the service's mappings,** so revoke and clearing need a rule of their own ([0170](0170-dma-quarantine.md)).
- **Physical addresses stay hidden from other programs.** Only holders of the DMA capability can learn them: the IOMMU service, and on machines without an IOMMU the drivers that do DMA ([0171](0171-no-iommu-trusted-drivers.md)).
- **It runs before any driver that uses DMA,** so it's in the ramdisk ([0155](0155-ramdisk-reaches-storage.md)).
- **The IOMMU's registers are the service's alone.** No driver gets a capability to them ([0172](0172-device-manager.md)).
- **06's question on protecting against DMA is answered,** together with [0170](0170-dma-quarantine.md) and [0171](0171-no-iommu-trusted-drivers.md).
