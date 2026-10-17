# 0199. thisOS is licensed under MIT OR Apache-2.0

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

std's platform layer for thisOS is meant to go upstream ([0187](0187-pinned-stable-toolchain.md)), and rust-lang/rust takes code only under MIT OR Apache-2.0. The system library is mapped into every program ([0117](0117-system-library.md)), and the `thisos` crate and the generated bindings are compiled into third-party programs ([0178](0178-native-api-crate.md), [0195](0195-bindings-safe-output.md)). The `bootloader` crate, which we fork ([0150](0150-bootloader-aarch64.md)), is under MIT or Apache-2.0. seL4's kernel is under GPL-2.0, and Hubris is under MPL-2.0.

## Options

1. **MIT OR Apache-2.0** for everything.
2. **A copyleft kernel,** under GPL-2.0 or MPL-2.0, with everything programs link or load under MIT OR Apache-2.0.
3. **MPL-2.0** for the whole OS.
4. **Apache-2.0 only.**

## Decision

Option 1.

- All of thisOS is licensed under MIT OR Apache-2.0, at the recipient's option: the dual license Rust itself uses.
- That covers the kernel, the system library, std's patch, the services, the tools and the documents in the repository.

## Consequences

- **Third-party programs owe nothing beyond keeping notices** for the parts of thisOS they contain.
- **std's patch can go upstream as it is** ([0187](0187-pinned-stable-toolchain.md)).
- **Our fork of the `bootloader` crate keeps its license,** which is the same ([0150](0150-bootloader-aarch64.md)).
- **Anyone may build closed products on thisOS,** and changes to it needn't be shared.
- **Dependencies must allow shipping under these terms,** which the dependency check enforces ([0193](0193-vendored-reviewed-crates.md)).
- **07's question on the license is answered.**
