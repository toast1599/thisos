# 0003. Rust is the system language; the Rust standard library replaces libc

- **Status:** Accepted, amended by [0082](0082-corrections.md) and [0221](0221-corrections.md)
- **Date:** 2026-10-14

## Context

Traditional operating systems are built around C and libc. libc defines both the system's language and its programming interface.

## Options

1. C and libc, the traditional approach.
2. Rust, with std running on top of a libc that is itself written in Rust. Redox started with its own std platform layer and later moved to this model (relibc), largely for compatibility with existing software.
3. Rust, with std ported directly to the OS and no libc. Hermit, Xous and Fortanix SGX already provide std this way.

## Decision

Option 3. Rust is the system language, playing the role C plays in traditional systems. Programs target the Rust standard library, which is ported to the OS in stages: `core`, then `alloc`, then `std`. There is no libc. Dropping C removes the legacy baggage.

## Consequences

- We write std's platform layer for this OS.
- Building std requires nightly Rust with `-Zbuild-std`. That stays true even after the target is accepted upstream at the lowest tier, so the toolchain has to be pinned ([0005](0005-unsafe-code-policy.md)).
- Rust has no stable ABI, so the OS ABI has to be defined explicitly ([0007](0007-abi-calls-and-interfaces.md)).
- With no stable ABI, every program statically links its own copy of std. A security fix in std means rebuilding and re-verifying every program, and that update mechanism still has to be designed.
- Crates that depend on the `libc` crate or on C libraries (`-sys` crates) won't work until they are ported.
- Self-hosting is unsolved: rustc generates code through LLVM, which is written in C++. Building Rust on the OS itself needs an answer later.
