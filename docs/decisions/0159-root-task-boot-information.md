# 0159. The root task receives the usual startup data, with a block of boot information

- **Status:** Accepted, amended by [0213](0213-power-capability.md)
- **Date:** 2026-10-17

## Context

Every program's startup data is a versioned block defined in the IDL ([0146](0146-startup-data-block.md)). The root task is the one program the kernel builds ([0052](0052-user-mode-page-tables.md)), and the kernel may write its memory at boot to set it up ([0072](0072-registers-only.md)). It receives every capability that exists at boot ([0024](0024-root-task.md)).

## Options

1. **The same block, plus boot information** defined in the IDL.
2. **A private boot-information page,** like seL4's bootinfo frame, in a format shared only by the kernel and the root task.

## Decision

Option 1.

- The kernel writes the root task's startup data in the same block every program receives ([0146](0146-startup-data-block.md)).
- It adds a boot-information block, also defined in the IDL. That block holds the memory map, the address of the ACPI tables or the device tree, the boot settings ([0157](0157-boot-settings-in-ramdisk.md)), a random seed from the firmware ([0174](0174-random-service-and-seed.md)), the ramdisk and the framebuffer, and the index and kind of every initial capability.
- The initial capabilities are:
  - memory capabilities for all RAM the kernel doesn't use, including the memory the ramdisk was loaded into;
  - device-memory capabilities for the rest of the physical address range ([0168](0168-device-memory-and-ports.md)), except the kernel's interrupt controller, timer and serial port ([0152](0152-kernel-log-and-reset.md));
  - the interrupt-control capability ([0167](0167-interrupts-as-notifications.md)), and on x86_64 the I/O-port capability ([0168](0168-device-memory-and-ports.md));
  - the capability for creating programs ([0031](0031-user-mode-loader.md), [0083](0083-program-is-address-space.md)), the scheduling-control capability ([0134](0134-scheduler-interface.md)), the counter capability ([0113](0113-counter-capability.md)), every priority capability ([0033](0033-cpu-fairness.md), [0090](0090-priorities-in-kernel.md)), the core-start capability and the boot core's capability ([0165](0165-core-objects.md)), and the DMA capability ([0169](0169-iommu-service.md));
  - capabilities to the root task's own address space, capability space and first thread ([0108](0108-startup-information.md));
  - the ramdisk and the kernel's log as read-only pages ([0152](0152-kernel-log-and-reset.md), [0153](0153-build-unpacks-ramdisk.md)), and the system library's pages as read-and-execute pages ([0154](0154-library-entry-table.md)). Being pages already, they aren't cleared when they're mapped ([0051](0051-kernel-clears-memory.md)).

## Consequences

- **std starts the root task like any program.** It reads the same block, and only the root task reads the boot information.
- **The boot information grows like any interface** ([0143](0143-add-only-versioning.md)).
- **The kernel writes this startup data, not the loader** ([0107](0107-loader-writes-startup-data.md)), just as it builds the root task's address space ([0052](0052-user-mode-page-tables.md)).
- **The root task sees physical addresses** in the memory map. It's in the trusted base already ([0024](0024-root-task.md)).
- **The kernel's own devices stay the kernel's.** No capability reaches the interrupt controller, the timer or the serial port it uses.
