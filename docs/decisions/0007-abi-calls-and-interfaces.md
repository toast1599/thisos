# 0007. ABI: C calling convention, interfaces defined in an IDL, add-only versioning

- **Status:** Superseded by [0144](0144-c-calls-and-what-crosses.md) (never accepted)
- **Date:** 2026-10-14

## Context

Rust doesn't guarantee type layout or calling convention from one compiler version to the next. Only `#[repr(C)]` and `extern "C"` are stable. The ABI is frozen at the end of Week 1, so it can't be defined as "Rust types". It has to be a binary interface that this project defines itself.

The architect is open on this decision and asked for a recommendation.

## Options

1. C calling convention, plus hand-written `#[repr(C)]` structs with documented rules.
2. C calling convention, plus interfaces defined in our own IDL, modeled on Fuchsia's FIDL.
3. C calling convention, plus WIT, the IDL of the WebAssembly component model.
4. A proposed stable Rust ABI (`extern "crabi"`). It isn't stable, so it can't be the basis of a frozen ABI.

## Proposed decision

1. **Calls.** Calls use the platform's C calling convention for each supported architecture, through `extern "C"`. On x86_64 that's System V AMD64; on AArch64 it's AAPCS64.
   - Programs in kernel mode call kernel entry points as ordinary function calls. Hermit already does this.
   - Three rules apply on top of the convention:
     - strings are UTF-8, passed as a pointer plus a length, with no NUL terminator;
     - errors are returned as values, with no errno;
     - there are no variadic functions.
2. **What crosses a boundary.** Only these may cross an interface:
   - fixed-layout structs
   - pointer + length buffers
   - typed handles

   `String`, `Vec`, `Box`, trait objects, generics and enums that carry data never cross. Memory is freed by whoever allocated it.
3. **Interface definitions.** One IDL is the single source of truth for every interface, both the kernel API and the protocols between programs. Three things are generated from it:
   - the fixed-layout types
   - the safe Rust bindings, audited once as part of the trusted set ([0005](0005-unsafe-code-policy.md))
   - the reference documentation
4. **Versioning.** The versioning rules are part of the frozen design:
   - interfaces only grow, with new functions and fields appended at the end;
   - existing functions and fields never change meaning;
   - every interface carries a version number;
   - a tool checks backward compatibility automatically.

   The freeze covers these rules and version 1 of the core interfaces. The IDL tooling is built after the freeze.
5. **Which IDL.** Recommended: our own, modeled on FIDL, which was designed for native code, handles, in-place decoding and versioning. The alternative is WIT, which would let the future WebAssembly runtime ([0006](0006-non-rust-software.md)) offer the same interfaces to wasm programs. Its costs are that WIT's binary layout is defined only for wasm memory, and that WIT is an external spec that keeps changing. Not yet decided.

## Consequences

- Freezing the ABI means freezing the rules plus version 1. Everything after that is additive.
- The calling convention for each architecture follows from the target-architecture decision ([01-objective](../design/01-objective.md)).
