# 0224. On the laptop, thisOS keeps its storage on a USB disk, and never opens the internal disk

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The laptop boots each checkpoint's image from a USB stick ([0223](0223-qemu-and-laptop-checkpoints.md)). Its only internal disk, an NVMe disk, holds two other operating systems and has no free space, as checked on 2026-10-19. The ramdisk holds the drivers that storage depends on ([0155](0155-ramdisk-reaches-storage.md)), and a bug in a storage driver can write anywhere on its disk. QEMU emulates NVMe disks, xHCI controllers and USB storage.

## Options

1. **A USB disk on the laptop, and NVMe in QEMU.** thisOS never opens the laptop's internal disk.
2. **RAM-backed storage on the laptop.** Laptop images carry the whole system image in the ramdisk, with a RAM-backed block device.
3. **A partition on the internal disk,** with NVMe in both places.
4. **No storage on the laptop.** Checkpoints after the storage steps stop at the ramdisk's programs there.

## Decision

Option 1.

- In QEMU, the system's storage is an NVMe disk, the kind most real PCs have.
- On the laptop, thisOS keeps its storage on a USB disk, through xHCI and USB mass-storage drivers.
- Laptop images start no driver for the internal disk.
- QEMU tests the USB drivers too, with its xHCI controller and USB storage.

## Consequences

- **The laptop's other systems are never at risk** from thisOS's storage code.
- **The roadmap gains a USB step** ([0227](0227-steps-and-phases.md)): xHCI, the USB core, mass storage and keyboards.
- **The NVMe driver is the one real PCs need,** and QEMU tests it on both architectures.
- **The ramdisk holds both sets of storage drivers,** and the device manager starts those a machine needs ([0155](0155-ramdisk-reaches-storage.md), [0172](0172-device-manager.md)).
