# 0006. Non-Rust software is supported later; route still open

- **Status:** Accepted, amended by [0082](0082-corrections.md)
- **Date:** 2026-10-14

## Context

A general-purpose OS ([0001](0001-general-purpose-os.md)) will eventually need to run software written in other languages, such as Python, JavaScript, Go and C/C++. Under [0002](0002-kernel-mode-verified-execution.md), [0003](0003-rust-std-no-libc.md) and [0005](0005-unsafe-code-policy.md), only verified safe Rust runs natively.

## Options

1. Never support it.
2. Run it in a sandbox, for example a WebAssembly runtime written in Rust.
3. Run it in a separate compatibility area isolated by hardware.
4. Run it on Rust implementations of the other languages' runtimes.

## Decision

Non-Rust software will be supported (not option 1). It is later work and not part of the initial system. The route is still open. The most likely route is a Rust implementation, such as a WebAssembly runtime written in Rust, but this isn't locked in.

## Consequences

- Interpreters written in safe Rust fit the model as it stands, because they're ordinary safe programs.
- Some runtimes generate machine code while running: JIT compilers, or compiling WebAssembly to native code. rustc never checked that code, so the code generator has to belong to the trusted set ([0005](0005-unsafe-code-policy.md)). It is still undecided whether the frozen design should reserve a place for this now, so that adding a fast runtime later doesn't break the freeze ([02-verification](../design/02-verification.md)).
