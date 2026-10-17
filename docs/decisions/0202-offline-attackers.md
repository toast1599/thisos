# 0202. Attackers who get the machine while it's off are in scope

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The threat model puts speculative leaks and denial of service between programs in scope, and leaves Rowhammer and the firmware out ([0016](0016-threat-model-scope.md)). It doesn't say whether someone with physical access is in scope. Yet the boot loader checks the kernel and the ramdisk against the system image's signature ([0156](0156-boot-loader-checks-image.md)), and swapped pages are encrypted with a key that dies at shutdown ([0077](0077-swap-in-default-pager.md)). Stored files are kept in the clear.

## Options

1. **In scope, with stored data encrypted later.** Verified boot covers the system's integrity now, and the filesystem encrypts stored data later.
2. **Out of scope,** documented as a limitation.
3. **In scope, with full-disk encryption now,** its key sealed to the machine's TPM.

## Decision

Option 1.

- An attacker who gets the machine while it's off is in scope: someone who reads or rewrites its disk, or replaces its boot partition.
- Verified boot protects the system's integrity now ([0156](0156-boot-loader-checks-image.md)).
- Stored data will be encrypted by the filesystem, with a key for each user that the login service unlocks at login. That comes later, with the filesystem and the login service ([0029](0029-users-as-capabilities.md), [0030](0030-filesystem-decides-access.md)).
- Attacks on a running machine's hardware, such as reading its memory after a cold reset or probing its buses, stay out of scope, like Rowhammer.
- This amends 0016: its list of threats in scope gains this attacker, and its list out of scope gains attacks on a running machine's hardware.

## Consequences

- **Older signed images are a known gap.** An older signed image or boot loader still boots ([0209](0209-no-rollback-protection.md)).
- **Data that belongs to no user needs a key too,** such as the login service's records and the owner's approvals of signers ([0206](0206-owner-approves-signers.md)). A key bound to the machine, for example one sealed to its TPM, is designed with the same later work. Until then, an attacker with the disk can change such data.
- **Packages resist tampering already.** The package store checks each manifest's signature after every boot, and every block against its hash as it's read ([0203](0203-package-format.md), [0207](0207-install-and-start-checks.md)).
- **Swap encryption fits this threat model:** swapped pages can't be read after shutdown ([0077](0077-swap-in-default-pager.md)).
- **Devices plugged into a running machine are a different threat,** which the IOMMU covers ([0169](0169-iommu-service.md), [0214](0214-interrupt-remapping.md)).
- **The design of the encryption keys is later work,** on Day 8's list ([08-roadmap](../design/08-roadmap.md)).
