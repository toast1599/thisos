# 06. Boot, user-mode drivers, std mapping, non-Rust software

- **Day:** 6 (Sat 2026-10-17)
- **Status:** In progress

## Covers

How the system starts, how it talks to hardware, and how the Rust standard library maps onto the OS.

## Decided

- [0004](../decisions/0004-not-unix.md) Not Unix-like: `target_family` is not `unix`
- [0006](../decisions/0006-non-rust-software.md) Non-Rust software is supported later; route still open (amended by [0082](../decisions/0082-corrections.md))
- [0015](../decisions/0015-clean-apis.md) APIs and ABIs are designed for this system: inspired by others, never bound to them
- [0024](../decisions/0024-root-task.md) At boot, one root task receives every initial capability
- [0032](../decisions/0032-minimal-hardware-discovery.md) The kernel reads only the hardware information it needs to start (amended by [0152](../decisions/0152-kernel-log-and-reset.md))
- [0071](../decisions/0071-three-memory-programs.md) The root task, the memory service and the default pager are three separate programs
- [0097](../decisions/0097-user-mode-starts-cores.md) User mode finds the CPUs, and the root task asks the kernel to start each one
- [0131](../decisions/0131-namespace-path-prefixes.md) A namespace maps path prefixes to capabilities
- [0145](../decisions/0145-std-over-native-api.md) std is a layer over the native API
- [0146](../decisions/0146-startup-data-block.md) Startup data is a versioned block defined in the IDL

The boot path:

- [0149](../decisions/0149-bootloader-crate-uefi.md) The kernel is booted by the `bootloader` crate, through UEFI only
- [0150](../decisions/0150-bootloader-aarch64.md) AArch64 support is added to the `bootloader` crate
- [0151](../decisions/0151-boot-info-from-crate.md) The kernel starts from the crate's boot information
- [0152](../decisions/0152-kernel-log-and-reset.md) The kernel writes its log to memory and a serial port, and resets the machine itself
- [0153](../decisions/0153-build-unpacks-ramdisk.md) The build unpacks the root task and the system library into the ramdisk
- [0154](../decisions/0154-library-entry-table.md) Programs call the system library through a table of entry points
- [0155](../decisions/0155-ramdisk-reaches-storage.md) The ramdisk holds the programs needed to reach storage
- [0156](../decisions/0156-boot-loader-checks-image.md) The boot loader checks the kernel and the ramdisk against the system image's signature
- [0157](../decisions/0157-boot-settings-in-ramdisk.md) Boot settings live in the ramdisk's header
- [0158](../decisions/0158-image-try-count.md) A new system image is tried a limited number of times, then the boot loader falls back

Starting the system:

- [0159](../decisions/0159-root-task-boot-information.md) The root task receives the usual startup data, with a block of boot information
- [0160](../decisions/0160-root-task-starts-four.md) The root task starts four programs itself, and the rest from a boot manifest
- [0161](../decisions/0161-root-task-before-scheduler.md) The root task runs alone until it starts the scheduler
- [0162](../decisions/0162-scheduler-above-priorities.md) A core's scheduler thread runs above every priority, by a rule in the kernel
- [0163](../decisions/0163-services-hand-first-capabilities.md) A service hands the first capabilities it issues to the program that started it
- [0164](../decisions/0164-root-task-memory-pool.md) The root task keeps back a pool of memory for the programs it pages
- [0165](../decisions/0165-core-objects.md) Each core is a kernel object, named by its capability
- [0166](../decisions/0166-core-revoke-offline.md) Revoking a running core's memory takes the core offline

Drivers and devices:

- [0167](../decisions/0167-interrupts-as-notifications.md) Interrupts reach drivers as notifications
- [0168](../decisions/0168-device-memory-and-ports.md) Device memory and I/O ports are capabilities of their own
- [0169](../decisions/0169-iommu-service.md) A user-mode IOMMU service keeps the devices' I/O page tables
- [0170](../decisions/0170-dma-quarantine.md) Memory mapped for a device is held back after a revoke, until the IOMMU service releases it
- [0171](../decisions/0171-no-iommu-trusted-drivers.md) Without an IOMMU, the drivers of DMA-capable devices are trusted
- [0172](../decisions/0172-device-manager.md) A device manager starts each driver with exactly its device's capabilities
- [0173](../decisions/0173-time-service-entry.md) Programs reach the time service through the namespace entry `svc/time`
- [0174](../decisions/0174-random-service-and-seed.md) Random numbers come from a random service, and every program starts with a seed

The std mapping:

- [0175](../decisions/0175-rust-target-names.md) The Rust targets are `x86_64-unknown-thisos` and `aarch64-unknown-thisos`, with no target family
- [0176](../decisions/0176-paths-and-current-directory.md) Paths that start with `/` resolve through the namespace, and others through a current directory
- [0177](../decisions/0177-locks-on-parking.md) std's locks are built on parking, and parking on notifications
- [0178](../decisions/0178-native-api-crate.md) The native API is a separate crate
- [0179](../decisions/0179-command-copies-namespace.md) std's `Command` gives a child a copy of the spawner's namespace by default
- [0180](../decisions/0180-streams-and-exit-status.md) Standard streams use a byte-stream protocol, and a program ends with a code or an abnormal end
- [0181](../decisions/0181-network-service.md) `std::net` uses a network service through the namespace
- [0182](../decisions/0182-all-of-std.md) All of std is supported, in stages, and allocation failure aborts

Storage and non-Rust software:

- [0183](../decisions/0183-access-on-directories.md) Stored access information lives on directories, as grants to stable user IDs
- [0184](../decisions/0184-shared-file-pages.md) Read-only file pages are shared from the service's cache, which pays for them
- [0185](../decisions/0185-swap-partition.md) Swap goes to a partition, through the block-storage service
- [0186](../decisions/0186-non-rust-route.md) Non-Rust software runs on WebAssembly first, then natively on the native API

## Open questions

None are left in this document. Questions that depend on other topics are listed with those topics:

- **Signing** ([02-verification](02-verification.md)): who signs the system image and the boot loader, and how the boot loader stores the key it checks against [0156](../decisions/0156-boot-loader-checks-image.md). Also how the boot loader itself is updated [0158](../decisions/0158-image-try-count.md).
- **Packages** ([02-verification](02-verification.md)): the list in the ramdisk that names the system image's other packages by hash ([0155](../decisions/0155-ramdisk-reaches-storage.md), [0156](../decisions/0156-boot-loader-checks-image.md)).
- **Build system** ([07-engineering](07-engineering.md)): the image builder, which unpacks programs into the ramdisk and writes its header ([0153](../decisions/0153-build-unpacks-ramdisk.md), [0157](../decisions/0157-boot-settings-in-ramdisk.md), [0160](../decisions/0160-root-task-starts-four.md)), and the pinned `bootloader` crate and our fork of it ([0149](../decisions/0149-bootloader-crate-uefi.md), [0150](../decisions/0150-bootloader-aarch64.md)).
- **Roadmap** ([08-roadmap](08-roadmap.md)): when the crate's AArch64 support, signature check and image selection are built ([0150](../decisions/0150-bootloader-aarch64.md), [0156](../decisions/0156-boot-loader-checks-image.md), [0158](../decisions/0158-image-try-count.md)), and when the device manager, the IOMMU service, the time, random and network services, the std stages and the WebAssembly runtime are built.
