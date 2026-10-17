# 0187. The toolchain is a pinned stable Rust release, and std's platform layer is a patch over its library

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The kernel is built with one pinned, audited toolchain ([0037](0037-kernel-toolchain.md)). 0037 covered only the kernel, but std stays in the audited set ([0017](0017-unsafe-code-kernel-and-services.md)), and every program in the system image links its own copy of it ([0003](0003-rust-std-no-libc.md), [0044](0044-system-image-is-os-shipped.md)). The `bootloader` crate is pinned as the kernel's toolchain is ([0149](0149-bootloader-crate-uefi.md)). Building std for a new target needs `-Zbuild-std`, a nightly feature ([0003](0003-rust-std-no-libc.md)). Until rust-lang/rust accepts the targets `x86_64-unknown-thisos` and `aarch64-unknown-thisos` ([0175](0175-rust-target-names.md)), std's platform layer for thisOS needs a home.

Checked on 2026-10-18, the current stable release is 1.99.1. Its Cargo refuses `-Zbuild-std` on the stable channel, but accepts it with `RUSTC_BOOTSTRAP=1`. Rust for Linux builds the Linux kernel with stable releases in the same way.

## Options

1. **A pinned stable release, with a library patch.** The compiler is upstream's, unchanged. `RUSTC_BOOTSTRAP=1` unlocks `-Zbuild-std` and the few other `-Z` flags the build needs. The targets are JSON target specs, and std's platform layer is a patch over that release's library sources.
2. **A pinned nightly,** with the same library patch.
3. **Our own fork of rust-lang/rust,** with built-in targets, built and shipped by the project.
4. **Upstream from the start.** The targets go to rust-lang/rust as tier 3 at once, and std's platform layer is written there, on a pinned nightly.

## Decision

Option 1.

- One pinned stable Rust release builds everything the project ships: the kernel, std, every program in the system image, the boot loader, and the tools the build runs. This extends 0037, which pinned only the kernel's toolchain.
- The compiler and Cargo are upstream's binaries, unchanged. The build sets `RUSTC_BOOTSTRAP=1` to unlock `-Zbuild-std` and the other `-Z` flags it needs.
- The targets `x86_64-unknown-thisos` and `aarch64-unknown-thisos` ([0175](0175-rust-target-names.md)) are JSON target specs, kept in the repository.
- std's platform layer for thisOS is a patch over the pinned release's library sources, kept in the repository. `-Zbuild-std` builds std from the patched sources.

## Consequences

- **Releases are better tested than nightlies.** Each stable release has had six weeks of beta testing, with fixes backported. That matters for miscompilations in the kernel ([0037](0037-kernel-toolchain.md)).
- **The compiler isn't ours to fix.** A compiler bug is fixed by moving to a release that fixes it ([0188](0188-upgrade-every-second-release.md)). Only the library sources carry our changes.
- **`RUSTC_BOOTSTRAP` gives up stable's compatibility promise.** The unstable flags can change between releases. Pinning absorbs that, since each upgrade checks them ([0188](0188-upgrade-every-second-release.md)).
- **Unstable language features stay out of most crates.** Every crate outside the audited set gets an empty list of allowed features, whatever `RUSTC_BOOTSTRAP` permits ([0191](0191-audited-list-and-wrapper.md)).
- **0003's need for nightly features is met on a stable release** ([0221](0221-corrections.md)).
- **Third-party developers build with the same pieces.** For each pinned release, the project publishes the target specs and std's patch, to use with that upstream release. Programs reach the system only through the system library's stable interface ([0117](0117-system-library.md)), so they needn't use the release the OS itself is built with.
- **The patch stays within what upstream would accept,** so the targets can be offered to rust-lang/rust later ([0178](0178-native-api-crate.md)). The roadmap says when ([08-roadmap](../design/08-roadmap.md)).
- **07's question on the toolchain is answered,** together with [0188](0188-upgrade-every-second-release.md) and [0189](0189-kernel-none-targets.md).
