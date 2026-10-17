# 0155. The ramdisk holds the programs needed to reach storage

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The `bootloader` crate loads the kernel and one ramdisk from the boot partition ([0149](0149-bootloader-crate-uefi.md)). Nothing else is in memory until user mode reads storage. Installed programs, the OS's own included, are immutable packages in a package store ([0041](0041-immutable-packages.md)), and the system image updates as one unit ([0042](0042-updates.md)). Fuchsia loads a small boot filesystem, bootfs, into memory, and everything else from storage.

## Options

1. **The programs needed to reach storage.**
2. **The whole system image.**

## Decision

Option 1.

- The ramdisk holds the root task and the system library, already unpacked ([0153](0153-build-unpacks-ramdisk.md)).
- It holds the programs needed to reach storage: the scheduler, the memory service, the default pager, the loader, the hardware-discovery service, the device manager, the IOMMU service, the drivers that storage depends on, the block-storage service, the filesystem and the package store. It also holds the services these use: the time and random services ([0173](0173-time-service-entry.md), [0174](0174-random-service-and-seed.md)).
- It also holds the boot manifest ([0160](0160-root-task-starts-four.md)), the boot settings ([0157](0157-boot-settings-in-ramdisk.md)), and the list of the system image's other packages with their hashes ([0156](0156-boot-loader-checks-image.md)).
- Everything else in the system image is loaded from storage, through the package store.

## Consequences

- **The ramdisk stays small** as the system image grows.
- **The rest of the image lives in the package store,** on the system's storage. The list in the ramdisk names each of its packages by hash ([0156](0156-boot-loader-checks-image.md)).
- **Memory the ramdisk takes up goes back to the memory service** once no running program uses it. The root task destroys those pages and passes the memory on, and the kernel clears it before it's used again ([0051](0051-kernel-clears-memory.md), [0159](0159-root-task-boot-information.md)).
- **The IOMMU service is in the ramdisk** because it has to run before any driver that uses DMA ([0169](0169-iommu-service.md)).
