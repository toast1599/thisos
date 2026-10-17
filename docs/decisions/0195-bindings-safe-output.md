# 0195. The build tool generates the bindings, and the generated code contains no unsafe code

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

Every interface is defined in our IDL, and the fixed-layout types, the safe Rust bindings and the reference documentation are generated from it ([0142](0142-own-idl.md)). The unsafe code in the bindings belongs to the audited set ([0142](0142-own-idl.md)). Build scripts run only in reviewed crates ([0192](0192-build-scripts-reviewed.md)). The `thisos` crate is generated from the IDL and published for third-party programs ([0178](0178-native-api-crate.md)).

## Options

1. **The build tool generates them, and the output is safe.** Generated code calls a small hand-written runtime crate for everything unsafe.
2. **Checked in.** Generated code is committed, and CI checks that it's up to date.
3. **Build scripts** run the IDL compiler in each crate.

## Decision

Option 1.

- The build tool runs the IDL compiler before Cargo builds anything ([0190](0190-cargo-and-build-tool.md)). Generated code is never checked in.
- Generated code contains no unsafe code. Everything that needs unsafe code, such as calls through the system library's entry table ([0154](0154-library-entry-table.md)), messages, regions and the accessors ([0133](0133-recovery-point-in-register.md)), is in a small hand-written runtime crate in the audited set ([0191](0191-audited-list-and-wrapper.md)).
- The published `thisos` crate ships its generated code, so third-party developers don't need the IDL compiler.

## Consequences

- **The IDL compiler needs no audit.** Its output passes the unsafe-code check like any other crate ([0191](0191-audited-list-and-wrapper.md)). A bug in it can produce wrong code, but not code that breaks memory safety.
- **0142's audited unsafe code is the runtime crate's.** It's written and audited once, not generated.
- **The IDL definitions are the only source.** Nothing generated can drift from them.
- **Reviewers read changes to the IDL,** not changes to generated code.
