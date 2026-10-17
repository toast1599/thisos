# 0172. A device manager starts each driver with exactly its device's capabilities

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Drivers learn about devices from a user-mode service that parses the ACPI tables or the device tree ([0032](0032-minimal-hardware-discovery.md)). Drivers run in user mode, and a failed driver is a failed program, which its supervisor can restart ([0014](0014-microkernel.md), [0105](0105-supervisor-reclaims.md)).

## Options

1. **A device manager.**
2. **Drivers ask the hardware-discovery service** for their devices.
3. **The root task starts the drivers** from the boot manifest, with fixed capabilities.

## Decision

Option 1.

- A device manager, started by the root task with the device-memory capabilities, except the IOMMU's registers ([0169](0169-iommu-service.md)), and the interrupt-control and I/O-port capabilities, matches the devices the hardware-discovery service finds to drivers.
- It starts each driver with capabilities for exactly its device: its memory ranges, its interrupts, and its DMA access through the IOMMU service ([0169](0169-iommu-service.md)).
- Bus drivers, such as the PCI driver, are drivers too. They report the devices they find back to the device manager.
- The device manager supervises the drivers, and restarts them when they fail ([0104](0104-supervisor.md), [0105](0105-supervisor-reclaims.md)).

## Consequences

- **A driver can reach only its own device.**
- **The parser of firmware tables decides no authority.** The hardware-discovery service only reports what it finds.
- **Devices found at run time,** such as hot-plugged PCI or USB devices, are handled like those found at boot.
- **The device manager joins the trusted base.** It holds the memory and interrupts of every device but the IOMMU. As an OS service, it's bound by [0017](0017-unsafe-code-kernel-and-services.md).
- **06's question on the driver model is answered,** together with [0167](0167-interrupts-as-notifications.md) to [0171](0171-no-iommu-trusted-drivers.md).
