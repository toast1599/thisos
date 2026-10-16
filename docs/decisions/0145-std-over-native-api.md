# 0145. std is a layer over the native API

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Programs target the Rust standard library ([0003](0003-rust-std-no-libc.md)). 05 narrowed std's role to two choices: std is the native API, or it's a thin, documented layer over the native API. Capabilities ([0020](0020-capabilities.md)) lean towards a layer, since std's paths, environment and process APIs assume ambient authority and have to be mapped ([06-system](../design/06-system.md)). The native API is now the system library ([0117](0117-system-library.md)) together with the bindings generated for services ([0142](0142-own-idl.md)).

## Options

1. **A layer over the native API.**
2. **std is the native API,** with extension traits for what std lacks.

## Decision

Option 1.

- The native API is the system library and the bindings generated from the IDL.
- std is a documented layer over it, which translates paths, environment variables, arguments and the like onto the native model ([0015](0015-clean-apis.md), [06-system](../design/06-system.md)).
- std never has abilities that the native API lacks.
- Programs that need capabilities, regions or notifications use the native API beside std.

## Consequences

- **The native model isn't bent to fit std** ([0015](0015-clean-apis.md)).
- **Most programs see only std.** The OS's own services and drivers use the native API directly.
- **std's platform layer is built on the bindings,** and std stays in the audited set ([0017](0017-unsafe-code-kernel-and-services.md)).
- **How std maps onto the native API** is designed with the std mapping ([06-system](../design/06-system.md)).
- **05's question on the role of std is answered.**
