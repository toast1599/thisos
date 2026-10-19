# 0261. The boot loader turns off bus mastering before it starts the kernel

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The IOMMU service confines DMA ([0169](0169-iommu-service.md)). It starts only after the scheduler, the memory service, the default pager, the loader, the time and log services and hardware discovery ([0160](0160-root-task-starts-four.md), [0227](0227-steps-and-phases.md)). Until then, nothing confines DMA. A device that the firmware left with bus mastering on can write into memory that the kernel has cleared and handed out ([0051](0051-kernel-clears-memory.md)), and devices plugged into a running machine are in scope ([0257](0257-outside-attackers.md)). The audit found, by searching the records, that none covers this window ([0256](0256-audit-findings-as-evidence.md)).

Turning off bus mastering in a device's configuration space stops the device from starting DMA. A malicious device can ignore that, and only an IOMMU that blocks DMA stops it. Blocking from the start needs IOMMU code in the boot loader: on SMMUv3, aborting while it's disabled, and on VT-d an empty root table, with the regions the firmware reserves for devices, such as VT-d's RMRR, still mapped. A display device's scanout is DMA too, and the kernel's framebuffer log depends on it ([0225](0225-framebuffer-log-at-failures.md)).

## Options

1. **Bus mastering off,** with blocking designed at step 16.
2. **The IOMMU blocks from handoff,** decided now.
3. **All of it designed at step 16.**
4. **The window accepted,** as a limitation.

## Decision

Option 1.

- Before it starts the kernel, our fork of the boot loader turns off bus mastering on every PCI device except display devices ([0150](0150-bootloader-aarch64.md)).
- The PCI bus driver turns it back on for a device when the device's driver gets DMA access ([0215](0215-pci-bus-driver-config.md)).
- A device that ignores this stays a known gap until the IOMMU service runs.
- Step 16 designs whether the IOMMU can block DMA from handoff, together with the regions the firmware reserves for devices ([0226](0226-open-topics-at-their-step.md)).
- This amends 0150, whose fork gains this change, and 0215, whose PCI bus driver turns bus mastering on.

## Consequences

- **DMA left running by the firmware can't reach reused memory.**
- **The framebuffer log keeps working,** since display devices are left alone ([0225](0225-framebuffer-log-at-failures.md)).
- **The boot loader writes configuration space,** in tier 1 code in our fork ([0242](0242-review-tiers.md)).
- **A malicious device still has a window** until the IOMMU service runs, unless step 16 closes it.
- **Step 16 is affected,** and builds the change in the fork ([08-roadmap](../design/08-roadmap.md)).
