# 0156. The boot loader checks the kernel and the ramdisk against the system image's signature

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The kernel, the boot path and the root task are the trusted base ([0014](0014-microkernel.md), [0024](0024-root-task.md)). Programs are signed ([0040](0040-signed-programs-owner-policy.md)), and the system image is built and signed as a whole ([0042](0042-updates.md)). The `bootloader` crate checks no signatures ([0149](0149-bootloader-crate-uefi.md)). Which keys sign is still open ([02-verification](../design/02-verification.md)).

## Options

1. **The boot loader checks both,** after Secure Boot has checked the boot loader.
2. **No checks at boot yet.** Checks start at the loader and the package store.
3. **Measured boot with a TPM.** The boot loader records hashes in the TPM, and nothing is refused at boot.

## Decision

Option 1.

- UEFI Secure Boot checks the boot loader.
- The boot loader checks the kernel and the ramdisk against the system image's signature before it starts the kernel. It doesn't start an image that fails.
- The package store checks the rest of the image's packages by hash, against the list in the ramdisk, which the boot loader's check already covers ([0041](0041-immutable-packages.md), [0155](0155-ramdisk-reaches-storage.md)).
- The check is added to the crate as an optional feature, and offered upstream along with the AArch64 work ([0150](0150-bootloader-aarch64.md)).

## Consequences

- **Every part of the image is checked before it runs,** in a chain that starts at the firmware.
- **The signature check runs in the trusted base.** Its unsafe code, if any, belongs to the audited set, like the rest of the crate ([0017](0017-unsafe-code-kernel-and-services.md), [0149](0149-bootloader-crate-uefi.md)).
- **Machines without Secure Boot still get the boot loader's check.** Only the boot loader itself goes unchecked.
- **Who signs the system image and the boot loader, and with which keys,** stays with signing ([02-verification](../design/02-verification.md)). The boot loader has to know the image's key, so how it's stored is part of that design.
- **02's question on where the check runs is answered for the system image.** For programs installed later, whether the loader or a separate service checks them stays open.
