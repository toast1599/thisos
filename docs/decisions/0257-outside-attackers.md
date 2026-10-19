# 0257. Attackers who reach a running machine from outside are in scope

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The threat model puts speculative leaks and denial of service between programs in scope ([0016](0016-threat-model-scope.md)), and attackers who get the machine while it's off ([0202](0202-offline-attackers.md)). The OS is built for programs that don't trust each other, web content and several users ([0001](0001-general-purpose-os.md)). The audit found, by searching the records, that none of them names an attacker who reaches a running machine through the network, or through a device plugged into it ([0256](0256-audit-findings-as-evidence.md)).

The network service parses packets from anywhere and holds every program's sockets ([0181](0181-network-service.md)). 0202 says devices plugged into a running machine are a threat that the IOMMU covers ([0169](0169-iommu-service.md), [0214](0214-interrupt-remapping.md)). The IOMMU confines what a device writes to memory and which interrupts it raises. It doesn't confine the descriptors and data that the USB stack parses from the device.

## Options

1. **Network and devices.**
2. **The network only.** Plugged-in devices stay as 0202 has them.
3. **Out of scope for the first release,** stated as a limitation. Its network runs only in QEMU ([0223](0223-qemu-and-laptop-checkpoints.md)).

## Decision

Option 1.

- Attackers who reach a running machine through the network, or through a device plugged into it, are in scope.
- Their input is untrusted wherever it's parsed: packets in the network service, descriptors and transfers in the USB stack, and the contents of any disk, which 0202 already treats as hostile.
- What a device writes to memory, and which interrupts it raises, stay confined by the IOMMU service ([0169](0169-iommu-service.md), [0214](0214-interrupt-remapping.md)).
- This amends 0016, whose threats in scope gain these attackers, and 0202, whose plugged-in devices are now a threat through their input as well as their DMA.

## Consequences

- **Code that parses outside input is reviewed in tier 2** ([0258](0258-outside-input-tier-2.md)), **and fuzzed** ([0259](0259-outside-input-fuzzed.md)).
- **What users type is part of the same boundary** ([0258](0258-outside-input-tier-2.md), [0260](0260-credentials-path.md)).
- **DMA before the IOMMU service runs is a gap,** which [0261](0261-bus-mastering-off-at-boot.md) narrows.
- **Steps 16, 17, 18, 21 and 22 are affected** ([08-roadmap](../design/08-roadmap.md)).
