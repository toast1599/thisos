# 0005. Unsafe code exists only in one closed, audited trusted set

- **Status:** Superseded by [0017](0017-unsafe-code-kernel-and-services.md)
- **Date:** 2026-10-14

## Context

Programs are safe because they're written in safe Rust. Every Rust system still contains `unsafe` code somewhere:
- `core`, `alloc` and `std` use it internally.
- The kernel and the std platform layer need it.
- Many popular crates contain it.

With everything running in kernel mode ([0002](0002-kernel-mode-verified-execution.md)), a bug in any unsafe code can compromise the whole machine.

## Options

1. Allow unsafe code in audited third-party crates, with an allowlist that grows over time.
2. Forbid unsafe code everywhere except in one closed trusted set.

## Decision

Option 2. All unsafe code lives in one closed trusted set. That code is written once, updated only periodically, and audited and tested heavily. Once the set is complete, no other unsafe code can be added anywhere, including programs and third-party crates.

## Evidence

Tested on 2026-10-14 with rustc 1.100.0-nightly (2026-09-01), edition 2024:

- `#![forbid(unsafe_code)]` rejects each of these:
  - `global_asm!`, including assembly that registers code to run at startup
  - `#[unsafe(no_mangle)]`
  - `#[unsafe(link_section = ...)]`
  - `unsafe extern` blocks, including ones that declare `safe fn`
- `-Zallow-features=` with an empty list rejects every unstable feature. It was tested with `specialization`, which is known to be unsound.
- The lint does not cover two things. Build scripts can link native object files or pass arbitrary linker arguments. Build scripts and procedural macros run arbitrary code at build time. Both are documented Cargo behavior and were not tested here.

## Consequences

- The trusted build system must:
  - apply `-F unsafe_code` and `-Zallow-features=` to every crate outside the trusted set;
  - forbid native linking from build scripts, or forbid build scripts entirely ([07-engineering](../design/07-engineering.md)).
- `core`, `alloc` and `std` belong to the trusted set, but they're written upstream and change with every release. The toolchain is therefore pinned, and every toolchain upgrade includes an audit of the changes to unsafe code.
- Crates that contain unsafe code can't be used as they are. Anything that genuinely needs unsafe must be offered by the trusted set as a safe API, or it won't exist on the OS. Examples:
  - hardware-accelerated cryptography
  - SIMD routines
  - lock-free concurrency
  - zero-copy conversion between bytes and structs
  - generated ABI bindings

  The list of trusted-set components is still open ([02-verification](../design/02-verification.md)).
