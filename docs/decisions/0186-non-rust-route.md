# 0186. Non-Rust software runs on WebAssembly first, then natively on the native API

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Non-Rust software is supported later, and its route was left open ([0006](0006-non-rust-software.md)). It can run natively in user mode ([0014](0014-microkernel.md)), and any compatibility it needs comes from a separate user-mode layer, never from the native API ([0015](0015-clean-apis.md)). JIT engines make memory executable with a capability ([0043](0043-executable-memory-capability.md), [0082](0082-corrections.md)). The system library uses the C calling convention ([0144](0144-c-calls-and-what-crosses.md)), and the IDL generates bindings ([0142](0142-own-idl.md)).

## Options

1. **WebAssembly first, then native programs,** both on the native API.
2. **A POSIX personality server,** which offers Unix semantics to ported C programs, as on QNX or MINIX.
3. **WebAssembly only.**

## Decision

Option 1.

- There are two routes, both on the native API.
- First, a WebAssembly runtime written in Rust, with WASI mapped onto the native API.
- Later, native programs in other languages, whose runtimes call the system library and bindings generated for their language from the IDL.
- Any POSIX compatibility is a library that such a program links, never a service or part of the native API. The OS still ships no libc ([0003](0003-rust-std-no-libc.md)).

## Consequences

- **The native API doesn't bend** to other languages ([0015](0015-clean-apis.md)).
- **The IDL compiler gains back ends** for other languages, later ([0142](0142-own-idl.md)).
- **A WebAssembly runtime that compiles to native code is a JIT engine,** and holds the capability from [0043](0043-executable-memory-capability.md).
- **C programs bring their own C library,** built on the native API.
- **0006's route is chosen, and 06's question on non-Rust software is answered.**
