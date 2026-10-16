# 0142. Interfaces are defined in our own IDL, modeled on FIDL

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

[0007](0007-abi-calls-and-interfaces.md) proposed one IDL as the single source of truth for every interface, and left open which one. The IDL now has to describe:

- messages of up to 8 words ([0119](0119-message-eight-words.md)) carrying up to 4 capabilities ([0123](0123-capabilities-in-messages.md));
- regions that are shared, lent or handed over ([0050](0050-clients-pay.md), [0058](0058-shared-and-handed-over.md));
- results that fill what the caller provides, then report what remains ([0139](0139-results-fill-and-report.md));
- the accessors that turn lost memory into errors ([0065](0065-lost-memory-is-an-error.md), [0133](0133-recovery-point-in-register.md)).

FIDL was designed for native code, handles and versioning, but its wire format is for Zircon's channels, with buffers of up to 64 KiB. WIT's binary layout is defined only for WebAssembly's linear memory.

## Options

1. **Our own, modeled on FIDL.**
2. **WIT,** the IDL of the WebAssembly component model.
3. **No IDL.** Hand-written `#[repr(C)]` types and Rust traits.

## Decision

Option 1.

- Every interface is defined in one IDL of our own, modeled on FIDL's ideas: protocols, typed capabilities with their rights, and availability by version.
- Its wire format is designed for messages in registers and for regions.
- It defines the system library's interface ([0117](0117-system-library.md)) as well as every interface between programs.
- Three things are generated from it: the fixed-layout types, the safe Rust bindings and the reference documentation.

## Consequences

- **We build the IDL compiler and the compatibility checker** ([0143](0143-add-only-versioning.md)). The roadmap has to place them ([08-roadmap](../design/08-roadmap.md)).
- **The generated bindings are OS code.** The unsafe code in them belongs to the audited set ([0017](0017-unsafe-code-kernel-and-services.md)).
- **The bindings hide the mechanics:** splitting data between messages and regions ([0119](0119-message-eight-words.md)), lending ([0050](0050-clients-pay.md)), the fill-and-report loop ([0139](0139-results-fill-and-report.md)) and recovery from lost memory ([0133](0133-recovery-point-in-register.md)).
- **We control how the IDL evolves,** which an external spec wouldn't allow.
- **The WebAssembly runtime gets its bindings later** ([0006](0006-non-rust-software.md)), generated from the same definitions.
- **05's question on the IDL is answered.**
