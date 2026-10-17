# 0157. Boot settings live in the ramdisk's header

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

After a kernel panic or a fault in the root task, the machine halts or reboots by a boot setting ([0110](0110-kernel-panic.md), [0115](0115-root-task-fault-boot-setting.md)). Later settings, such as requiring an IOMMU ([0171](0171-no-iommu-trusted-drivers.md)), would be security settings.

## Options

1. **In the ramdisk's header.**
2. **Compiled into the kernel.**
3. **An unsigned file** on the boot partition.

## Decision

Option 1.

- Boot settings are part of the fixed-layout header at the start of the ramdisk ([0153](0153-build-unpacks-ramdisk.md)).
- The kernel reads them at boot, and passes them to the root task in its boot information ([0159](0159-root-task-boot-information.md)).

## Consequences

- **Settings are covered by the image's signature** ([0156](0156-boot-loader-checks-image.md)). Changing one means building and signing a new image.
- **Development and production images differ only by their settings,** such as halting for inspection or rebooting after a panic ([0110](0110-kernel-panic.md)).
- **New settings are added to the header** by the kernel and the image builder, which change together ([0153](0153-build-unpacks-ramdisk.md)).
- **0110's and 0115's question on the boot setting is answered.**
