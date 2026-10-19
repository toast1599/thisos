# 0262. Interrupt entries check their source, and the configuration ports are never granted

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The IOMMU service maps a device's interrupts only to vectors whose interrupt capabilities its driver passes it ([0214](0214-interrupt-remapping.md)). On VT-d, a device picks its remapping entry by the value it writes. That value sits in its MSI-X table, inside the device memory its driver maps ([0168](0168-device-memory-and-ports.md)), which the PCI bus driver's mediation of configuration space doesn't reach ([0215](0215-pci-bus-driver-config.md)). A driver, or a device now in scope ([0257](0257-outside-attackers.md)), can write another device's entry. VT-d's entries refuse that only when their source check is on, and no record turns it on ([0256](0256-audit-findings-as-evidence.md)). On AArch64, the GIC's ITS takes a device's ID from the bus, not from what the device writes ([0214](0214-interrupt-remapping.md)).

On x86_64, I/O ports 0xCF8 to 0xCFF reach every device's configuration space, so a driver granted them could get around 0215. Step 16's attack test sends edu's own MSI, not a forged one ([08-roadmap](../design/08-roadmap.md)).

## Options

1. **Source checks, the ports never granted, and attack tests.**
2. **Attack tests only,** with no change to 0214 or 0215.
3. **Left as is.**

## Decision

Option 1.

- The IOMMU service turns on the source check in every interrupt-remapping entry it writes, so each entry accepts only the device it was made for.
- On x86_64, only the PCI bus driver holds I/O ports 0xCF8 to 0xCFF. The device manager never grants them to any other driver.
- Step 16 adds two attack tests: edu writing another device's entry, and a driver that asks for those ports.
- This amends 0214, whose remapping gains the source check, and 0215, whose PCI bus driver also holds the configuration ports.

## Consequences

- **A device can't raise another device's interrupts,** even by rewriting its own MSI-X table.
- **0215's mediation can't be bypassed** through I/O ports.
- **AArch64 needs nothing new,** since the ITS confines devices by construction.
- **Step 16 is affected** ([08-roadmap](../design/08-roadmap.md)).
