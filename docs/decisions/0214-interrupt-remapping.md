# 0214. Devices raise only the interrupts they were given, through interrupt remapping

- **Status:** Accepted, amended by [0262](0262-interrupt-source-check.md)
- **Date:** 2026-10-18

## Context

Interrupts reach drivers as notifications, and for an MSI vector, the kernel gives the address and data value that the device writes ([0167](0167-interrupts-as-notifications.md)). A device raises an MSI with an ordinary memory write. On x86_64, unless the IOMMU's interrupt remapping is on, a driver can make its device raise any vector on any core: other devices' vectors, the kernel's own, and exception vectors, whose handlers then misread the stack. That's a known way for a driver to take over a kernel (Wojtczuk and Rutkowska, 2011). The IOMMU service confines DMA ([0169](0169-iommu-service.md)), and drivers count as trusted only where there's no IOMMU ([0171](0171-no-iommu-trusted-drivers.md)). On AArch64, the GIC's ITS maps each device's messages only to the interrupts set up for that device, and the interrupt controller is the kernel's ([0167](0167-interrupts-as-notifications.md)).

## Options

1. **Remapping in the IOMMU service,** and the GIC's ITS on AArch64.
2. **Remapping in the kernel,** which gains some IOMMU code.
3. **No remapping.** The kernel's handlers tolerate stray interrupts.

## Decision

Option 1.

- The IOMMU service turns on interrupt remapping wherever the IOMMU offers it.
- It maps a device's interrupts only to vectors whose interrupt capabilities the driver passes it ([0167](0167-interrupts-as-notifications.md)), and gives the driver the address and data value the device writes. This amends 0167, whose kernel gives those values only where no IOMMU remaps interrupts.
- On AArch64, the kernel's interrupt-controller code maps each device's messages through the GIC's ITS, so a device raises only the interrupts made for it.
- Where neither exists, the drivers of DMA-capable devices count as trusted, as they do without an IOMMU. The boot setting that requires an IOMMU also requires this. This amends 0171, whose trusted drivers include these, and whose boot setting covers both.

## Consequences

- **A driver can't raise interrupts beyond its own.** Where there's an IOMMU, a driver is confined in both its DMA and its interrupts.
- **The IOMMU service learns each interrupt capability's vector** through the DMA capability, as it learns physical addresses ([0169](0169-iommu-service.md)).
- **The trusted base doesn't grow.** The IOMMU service already controls every device's DMA ([0169](0169-iommu-service.md)).
- **The ITS's tables are kernel objects,** paid for like any other ([0026](0026-programs-pay-kernel-objects.md), [0068](0068-no-kernel-heap.md)).
- **Both halves of what a device can write are confined:** memory and interrupts.
