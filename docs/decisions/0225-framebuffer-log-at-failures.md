# 0225. The kernel shows its log on the framebuffer when it fails, and while bring-up images start

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The kernel writes its log to a ring buffer in memory and to a serial port ([0152](0152-kernel-log-and-reset.md)). 0152 also considered the framebuffer that the boot information describes, and chose the serial port. The laptop has no serial port ([0223](0223-qemu-and-laptop-checkpoints.md)), so a kernel panic, a fault in the root task or a failed essential service would leave nothing visible there ([0110](0110-kernel-panic.md), [0115](0115-root-task-fault-boot-setting.md), [0216](0216-essential-service-failure.md)). The kernel maps no program data ([0055](0055-kernel-maps-only-objects.md)). Development and production images differ only by their boot settings ([0157](0157-boot-settings-in-ramdisk.md)).

## Options

1. **At failures, and in bring-up images.** The kernel draws its log on the framebuffer when it fails. With a boot setting, it also draws it as it's written, until the root task starts.
2. **Only at failures.**
3. **Never.** Only a user-mode console shows logs on the framebuffer.

## Decision

Option 1.

- When the kernel fails, it draws the end of its log on the framebuffer, after writing it to the serial port. That covers a kernel panic, a fault in the root task, and a fail through the power capability ([0110](0110-kernel-panic.md), [0115](0115-root-task-fault-boot-setting.md), [0213](0213-power-capability.md)).
- With a bring-up boot setting ([0157](0157-boot-settings-in-ramdisk.md)), the kernel also draws its log as it's written, until the root task starts.
- The kernel maps the framebuffer only while it draws.
- This amends 0152, whose log outputs gain the framebuffer at these times, and [0032](0032-minimal-hardware-discovery.md), whose list of what the kernel reads gains the framebuffer that the boot information describes ([0151](0151-boot-info-from-crate.md)).

## Consequences

- **Failures are visible on machines without a serial port.**
- **Screen contents stay out of the kernel's mappings** while the system runs ([0055](0055-kernel-maps-only-objects.md)). The kernel maps the framebuffer only before the root task starts, and after the system has failed.
- **The drawing code is generic.** A linear framebuffer needs no platform code ([0046](0046-scope-of-0008.md)), only a small built-in font.
- **A failure draws over the screen,** which user mode owns by then ([0159](0159-root-task-boot-information.md)). The system has stopped at that point anyway.
- **Bring-up images show where a hang happened,** if it happens before the root task starts.
