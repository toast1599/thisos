# 0190. Cargo builds thisOS, driven by a build tool written in Rust

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The build compiles for several targets: the kernel ([0189](0189-kernel-none-targets.md)), programs for the thisOS targets ([0175](0175-rust-target-names.md)), the boot loader for UEFI ([0149](0149-bootloader-crate-uefi.md)), and tools that run on the build machine. It runs the IDL compiler and the compatibility checker ([0142](0142-own-idl.md), [0143](0143-add-only-versioning.md)), and checks unsafe code, architecture-specific code and the kernel's stack depth. The image builder unpacks the first programs into the ramdisk and writes its header ([0153](0153-build-unpacks-ramdisk.md), [0157](0157-boot-settings-in-ramdisk.md), [0160](0160-root-task-starts-four.md)). The UEFI disk image is built with our fork of the `bootloader` crate ([0149](0149-bootloader-crate-uefi.md), [0150](0150-bootloader-aarch64.md)). Tests boot the system in QEMU. Hubris drives Cargo with a build tool written in Rust. Redox and Theseus use Make.

The `bootloader` crate's own build script builds its UEFI stage by running `cargo install`, which fetches the stage from crates.io unless it finds a local copy.

## Options

1. **Cargo, driven by a build tool written in Rust** inside the repository.
2. **Bazel or Buck2.**
3. **Make around Cargo.**

## Decision

Option 1.

- Cargo builds the code, in workspaces.
- A build tool written in Rust, inside the repository, drives every step: the toolchain settings ([0187](0187-pinned-stable-toolchain.md)), the IDL compiler ([0195](0195-bindings-safe-output.md)), the checks ([0191](0191-audited-list-and-wrapper.md), [0197](0197-architecture-check.md), [0198](0198-stack-depth-checker.md)), the image builder, signing ([0205](0205-root-and-working-keys.md)) and starting QEMU.
- The image builder is part of the build tool. It unpacks the root task, the system library and the four programs the root task starts into the ramdisk. It writes the ramdisk's header with the boot settings, the boot manifest, and the list of the image's other packages. It also lays out the boot partition of a new disk image ([0153](0153-build-unpacks-ramdisk.md), [0155](0155-ramdisk-reaches-storage.md), [0157](0157-boot-settings-in-ramdisk.md), [0160](0160-root-task-starts-four.md)).
- The build tool builds the boot loader's UEFI stage itself, from the vendored fork ([0193](0193-vendored-reviewed-crates.md)). The crate's build script, which runs `cargo install`, isn't used.

## Consequences

- **One language throughout,** and `-Zbuild-std` works as it is.
- **Build steps are Rust code,** which can be reviewed and tested like the rest.
- **The image builder is a build tool,** not part of the system image. It's built from the same source as the kernel, so the ramdisk's header can change with any image ([0221](0221-corrections.md)).
- **The build writes each program's declarations** from the interfaces it uses ([0147](0147-elf-declarations-only.md)), through the IDL compiler. It copies the first four programs' declarations into the boot manifest ([0160](0160-root-task-starts-four.md)).
- **Builds run offline** ([0192](0192-build-scripts-reviewed.md)).
- **Where each crate lives is Day 8's,** with the repository layout ([08-roadmap](../design/08-roadmap.md)).
- **07's question on the build system is answered,** together with [0191](0191-audited-list-and-wrapper.md) to [0198](0198-stack-depth-checker.md).
