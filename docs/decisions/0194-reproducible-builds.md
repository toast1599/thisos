# 0194. The system image and the boot loader build reproducibly

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The system image and the boot loader are signed ([0040](0040-signed-programs-owner-policy.md), [0156](0156-boot-loader-checks-image.md)). A signature shows who built them, not what source they were built from. The toolchain is pinned ([0187](0187-pinned-stable-toolchain.md)), and every dependency is vendored ([0193](0193-vendored-reviewed-crates.md)).

## Options

1. **Bit-for-bit reproducible.** The same source and toolchain give identical bytes on any machine.
2. **Not required.**

## Decision

Option 1.

- Building the same source with the same toolchain gives the same bytes for the system image and the boot loader, on any machine.
- The build records no timestamps, and no paths of the machine it ran on.
- The image builder writes the ramdisk and the boot partition's FAT filesystem in a fixed order, with fixed dates ([0190](0190-cargo-and-build-tool.md)).

## Consequences

- **Anyone can check a signed image against its source,** by building it and comparing.
- **Signing can happen apart from the build.** The key holder signs what a reproducible build produced ([0205](0205-root-and-working-keys.md)).
- **How reproducibility is checked,** for example by building twice in different places, is Day 8's, with testing ([08-roadmap](../design/08-roadmap.md)).
