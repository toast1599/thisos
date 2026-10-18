# 0223. thisOS is built and tested in QEMU, with a checkpoint on the development laptop at the end of each phase

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

QEMU runs the system with UEFI firmware: OVMF on x86_64, and AAVMF on AArch64 ([0149](0149-bootloader-crate-uefi.md), [0150](0150-bootloader-aarch64.md)). Checked on 2026-10-19 on the development laptop: QEMU 10.2.1 and OVMF, Secure Boot builds included, are installed, and KVM works. QEMU for AArch64 and AAVMF aren't installed yet ([0009](0009-x86-64-first.md)), but Ubuntu packages both. QEMU emulates the devices the design has to be tested on: Intel's IOMMU with interrupt remapping on q35, SMMUv3 and the GICv3's ITS on virt ([0169](0169-iommu-service.md), [0214](0214-interrupt-remapping.md)), SMT topologies ([0034](0034-smt-trusted-pairs.md)), and edu, a PCI device that does DMA and MSI.

The development laptop is a Lenovo IdeaPad Slim 5 14IMH9, with an Intel Core Ultra 5 125H. It has SMT on its performance cores only, 4-level paging only, VT-d with two IOMMU units, a PS/2 keyboard, an NVMe disk, xHCI USB, and Wi-Fi with no wired network. It has no serial port, so the kernel's log ([0152](0152-kernel-log-and-reset.md)) can't be read there. Secure Boot is off on it.

## Options

1. **QEMU only, with virtio devices.** Real hardware is later work.
2. **QEMU, plus the laptop at checkpoints.** The laptop boots each checkpoint's image as far as it can.
3. **QEMU, plus a real machine for each architecture,** with drivers for their devices.

For how the laptop runs checkpoints:

- **At the end of each phase,** from a USB stick.
- **After every step.**
- **Twice:** at the first user-mode program, and at the first release.

## Decision

Option 2, at the end of each phase.

- Every step is built and tested in QEMU: x86_64 on q35 with OVMF under KVM, and AArch64 on virt with AAVMF under emulation, each with and without an IOMMU.
- At the end of each phase, that phase's image boots on the laptop from a USB stick, and runs as far as the laptop's devices allow ([0227](0227-steps-and-phases.md)).
- The laptop has no wired network, so its network is later work ([0251](0251-later-work.md)). AArch64 runs only in QEMU.
- The console uses the framebuffer and a PS/2 keyboard, which QEMU's q35 has as well.
- Secure Boot is tested in QEMU. On the laptop it stays as its owner set it, and the boot loader's own check still runs ([0156](0156-boot-loader-checks-image.md)).
- Booted from removable media, the boot loader leaves the firmware's boot entries alone. This amends [0208](0208-boot-loader-beside-old.md), whose boot loader sets them up only for a system installed on the machine's own storage.

## Consequences

- **Real hardware is met early,** at seven checkpoints, without slowing the steps between them.
- **The laptop needs output without a serial port.** The kernel draws its log on the framebuffer when it fails ([0225](0225-framebuffer-log-at-failures.md)), and the log service shows logs there ([0236](0236-program-output-second-serial.md)).
- **Storage on the laptop needs care,** since its disk holds other systems ([0224](0224-laptop-usb-storage.md)).
- **Real firmware is met at the checkpoints,** such as the laptop's ACPI tables and the regions its firmware reserves for devices under VT-d. Each step designs what it needs of these ([0226](0226-open-topics-at-their-step.md)).
- **How the boot loader tells removable media apart** is designed at step 20 ([0226](0226-open-topics-at-their-step.md)).
- **CI runs in QEMU only** ([0237](0237-public-repository-and-ci.md)), and timing runs on the laptop ([0240](0240-timing-on-laptop.md)).
- **QEMU for AArch64 and AAVMF are installed at step 7** ([0009](0009-x86-64-first.md), [0150](0150-bootloader-aarch64.md)).
