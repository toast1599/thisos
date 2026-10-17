# 0150. AArch64 support is added to the `bootloader` crate

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The `bootloader` crate boots only x86_64 ([0149](0149-bootloader-crate-uefi.md)). AArch64 support is an open issue in its repository (#242), marked as wanting help. AArch64 is brought up together with the first user-mode program, around weeks 7 to 9 ([0045](0045-aarch64-with-first-user-program.md)).

## Options

1. **Extend the crate,** and offer the work upstream.
2. **Our own AArch64 UEFI stage,** which hands the kernel a `BootInfo` of the same shape.
3. **The Linux arm64 boot protocol,** with no boot loader: the kernel is loaded directly and given a device tree in a register.
4. **Limine on AArch64.**

## Decision

Option 1.

- We add AArch64 support to the `bootloader` crate, through UEFI.
- The work is offered upstream, and kept in a fork until it's merged.
- On AArch64, the boot information also carries the device tree's address.

## Consequences

- **One boot loader and one boot interface on both architectures.** The kernel's entry code differs by architecture ([0008](0008-architecture-neutral-design.md)), but what it receives doesn't.
- **We keep a fork until the work is merged.** Our other changes to the crate go the same way: the signature check ([0156](0156-boot-loader-checks-image.md)), image selection ([0158](0158-image-try-count.md)) and the random seed ([0174](0174-random-service-and-seed.md)).
- **The roadmap has to place the work** before AArch64's bring-up ([0045](0045-aarch64-with-first-user-program.md), [08-roadmap](../design/08-roadmap.md)).
- **QEMU for AArch64 needs UEFI firmware,** AAVMF, when it's installed ([0011](0011-aarch64-second.md)).
