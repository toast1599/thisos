# 0149. The kernel is booted by the `bootloader` crate, through UEFI only

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The boot path is in the trusted base, with the kernel and the hardware ([0014](0014-microkernel.md)). It has to start the kernel on x86_64, and on AArch64 from the first user-mode program on ([0009](0009-x86-64-first.md), [0011](0011-aarch64-second.md), [0045](0045-aarch64-with-first-user-program.md)). It also has to bring the root task into memory without the kernel parsing an executable ([0024](0024-root-task.md), [0031](0031-user-mode-loader.md)).

The `bootloader` crate from the rust-osdev project is a boot loader written in Rust. As checked on 2026-10-17, its latest release is 0.11.17, from 2026-07-27. It boots x86_64 through BIOS or UEFI, and loads the kernel's ELF file and one raw ramdisk from a FAT partition. It hands the kernel a `BootInfo` structure, defined in its `bootloader_api` crate. It supports no other architecture, and it checks no signatures.

## Options

1. **Our own UEFI boot stage,** written in Rust.
2. **The `bootloader` crate.**
3. **Limine,** written in C, for x86_64 and AArch64.
4. **GRUB with Multiboot2,** which is effectively x86-only.
5. **The kernel as a UEFI application.**

For firmware:

- **UEFI only.**
- **UEFI, with BIOS for development.**
- **Both.**

## Decision

Option 2, with UEFI only.

- The `bootloader` crate loads the kernel and the ramdisk, and starts the kernel.
- Only its UEFI path is used. No BIOS images are built.

## Consequences

- **We don't write a boot loader of our own.** The boot path is written in Rust, in a crate others maintain.
- **The crate joins the audited set.** The boot loader sits outside the system images it chooses between ([0158](0158-image-try-count.md)), but the project builds and ships it, and it runs in the trusted base. So [0017](0017-unsafe-code-kernel-and-services.md) applies to it as it does to the image ([0044](0044-system-image-is-os-shipped.md)), and the crate's unsafe code is brought into the audited set.
- **Its version is pinned, and every upgrade is reviewed,** as for the kernel's toolchain ([0037](0037-kernel-toolchain.md)).
- **AArch64 needs work on the crate** ([0150](0150-bootloader-aarch64.md)).
- **The crate checks no signatures,** so a check is added to it ([0156](0156-boot-loader-checks-image.md)).
- **The kernel builds its own page tables.** The crate's options to map all of physical memory, or to map page tables recursively, stay off, since the kernel maps only its own memory and kernel objects ([0055](0055-kernel-maps-only-objects.md), [0067](0067-object-window-offset.md)).
- **Secure Boot is available,** since it exists only on UEFI ([0156](0156-boot-loader-checks-image.md)).
- **QEMU runs the system with UEFI firmware,** OVMF on x86_64. Machines that boot only legacy BIOS aren't supported.
- **06's question on the boot loader is answered.**
