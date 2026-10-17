# 0215. The PCI bus driver keeps configuration space, and mediates drivers' access to it

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

A device manager starts each driver with exactly its device's capabilities, and bus drivers such as the PCI driver report the devices they find ([0172](0172-device-manager.md)). No record says who holds PCI configuration space. With raw access, a driver can move its device's memory ranges over another device's, or turn on the device's address translation (ATS), which lets the device get past the IOMMU ([0169](0169-iommu-service.md)). PCIe switches can also pass traffic between devices without it reaching the IOMMU, unless their access controls (ACS) are on.

## Options

1. **The PCI bus driver mediates.**
2. **Each driver gets its own** device's configuration space, as device memory ([0168](0168-device-memory-and-ports.md)).
3. **The device manager mediates.**

## Decision

Option 1.

- The PCI bus driver keeps every device's configuration space.
- Drivers reach their own device's configuration through it. It refuses writes that would move a device's memory ranges, redirect its interrupts, or turn on ATS.
- It turns on the access controls of PCIe switches, so that devices can't reach each other around the IOMMU.
- It joins the trusted base, beside the device manager ([0172](0172-device-manager.md)).

## Consequences

- **A driver can reach only its own device,** as 0172 intends, configuration included.
- **The IOMMU's confinement holds** ([0169](0169-iommu-service.md), [0214](0214-interrupt-remapping.md)).
- **Devices behind a switch without access controls can still reach each other,** so their drivers are trusted with each other's memory.
- **The PCI bus driver is an OS service,** bound by [0017](0017-unsafe-code-kernel-and-services.md). Storage depends on it, so it's in the ramdisk ([0155](0155-ramdisk-reaches-storage.md)).
