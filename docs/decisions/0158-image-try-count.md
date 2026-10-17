# 0158. A new system image is tried a limited number of times, then the boot loader falls back

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The system image switches atomically, with rollback ([0042](0042-updates.md)). The `bootloader` crate loads fixed file names from the boot partition ([0149](0149-bootloader-crate-uefi.md)).

## Options

1. **A try count with fallback.**
2. **UEFI boot entries,** one per image, with a trial boot through the firmware.
3. **A manual choice** at boot.

## Decision

Option 1.

- The boot partition holds the boot part of the current and the previous system image, each a kernel and a ramdisk, and a small record of which to boot.
- A new image is tried a limited number of times. Once the system is up, it's marked good.
- If it's never marked good, the boot loader goes back to the previous image.
- This is added to our fork of the crate, along with the signature check ([0156](0156-boot-loader-checks-image.md)).

## Consequences

- **A bad image can't keep a machine down.** The machine falls back without anyone at the console.
- **The update service marks an image good** once the system is up. It writes the record through the storage drivers and the boot partition's FAT filesystem.
- **No firmware runtime services are needed.** The record is an ordinary file.
- **Rollback also needs the previous image's packages,** which the package store keeps, named by hash ([0041](0041-immutable-packages.md)).
- **The boot loader itself sits outside the images,** so this rollback doesn't cover it. How the boot loader is updated is designed with signing and updates ([02-verification](../design/02-verification.md)).
