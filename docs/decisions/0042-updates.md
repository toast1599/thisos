# 0042. The system updates atomically as one image; third-party programs update package by package

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Each program statically links its own std ([0003](0003-rust-std-no-libc.md)), so a std fix reaches a program only when that program is rebuilt.

## Options

1. **An atomic system image,** with third-party programs updated per package.
2. **Everything per package,** the kernel included.
3. **Everything in one image,** installed programs included.

## Decision

Option 1.

- The kernel and the OS's own services update together as one versioned system image.
- The image is switched atomically, with rollback.
- Third-party programs update package by package.

## Consequences

- **The image is one unit.** It's what [0044](0044-system-image-is-os-shipped.md) counts as shipped by the OS, and it's built, signed and tested as a whole.
- **std fixes travel separately.** A std fix reaches the system with the next image, but reaches a third-party program only when its publisher rebuilds it. What the OS does about third-party programs built with a flawed std is open ([02-verification](../design/02-verification.md)).
- **Rollback needs the previous image kept,** which content-addressed packages make cheap ([0041](0041-immutable-packages.md)).
