# 0151. The kernel starts from the crate's boot information

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

[0032](0032-minimal-hardware-discovery.md) lets the kernel read only the memory map and information about the interrupt controller and a timer. The `bootloader` crate ([0149](0149-bootloader-crate-uefi.md)) hands the kernel a `BootInfo` structure with:

- the memory map;
- the address of ACPI's root table, the RSDP;
- a framebuffer, if there is one;
- the ramdisk's location and size;
- the kernel's own location, its stack and its thread-local template.

It names no interrupt controller, timer, serial port or reset method.

## Options

1. **The crate's boot information.** The kernel reads the interrupt controller and the timer from the firmware tables itself, as 0032 allows.
2. **A block from the boot stage.** The boot stage reads the firmware tables and hands the kernel everything it needs, so the kernel parses none.
3. **The kernel reads all of it,** including a serial port and the reset method.

## Decision

Option 1.

- The kernel's boot input is the crate's `BootInfo` ([0149](0149-bootloader-crate-uefi.md), [0150](0150-bootloader-aarch64.md)).
- The kernel reads the interrupt controller and the timer from ACPI's tables, through the RSDP that `BootInfo` gives, or from the device tree on AArch64 ([0150](0150-bootloader-aarch64.md)). 0032 stands for these.

## Consequences

- **The kernel parses a few firmware tables.** On x86_64, that's ACPI's interrupt-controller table (MADT). On AArch64, it's the device tree's or ACPI's description of the interrupt controller and the timer. This code is platform-specific ([0046](0046-scope-of-0008.md)).
- **The boot information is the crate's format.** It changes only when the crate changes, and the crate's version is pinned ([0149](0149-bootloader-crate-uefi.md)).
- **The kernel passes the firmware tables' address on** to the root task, for the hardware-discovery service ([0032](0032-minimal-hardware-discovery.md), [0159](0159-root-task-boot-information.md)).
- **The serial port and the reset method aren't in it,** so they're decided separately ([0152](0152-kernel-log-and-reset.md)).
