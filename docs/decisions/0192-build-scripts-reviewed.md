# 0192. Build scripts and procedural macros run only in reviewed crates, and never link native code

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

Build scripts and procedural macros run arbitrary code on the build machine at build time ([0005](0005-unsafe-code-policy.md)). Build scripts can also link native object files or pass arbitrary linker arguments, which `forbid(unsafe_code)` doesn't catch ([0005](0005-unsafe-code-policy.md)). 0005 left two choices: forbid native linking from build scripts, or forbid build scripts entirely ([07-engineering](../design/07-engineering.md)). What the build produces is signed ([0156](0156-boot-loader-checks-image.md)).

## Options

1. **Reviewed crates only, with no native linking.** Build scripts and procedural macros run only in reviewed crates. Native linking is refused outside the audited set, and builds run offline in a sandbox.
2. **No build scripts at all,** and procedural macros only from a short allowlist.
3. **No native linking only.** Build scripts may do anything else, and nobody reviews them.

## Decision

Option 1.

- Build scripts and procedural macros run only in crates that have passed review ([0193](0193-vendored-reviewed-crates.md)).
- The compiler wrapper ([0191](0191-audited-list-and-wrapper.md)) refuses native libraries and linker arguments from the build script of any crate outside the audited set.
- Builds run offline, in a sandbox that gives them only the source tree, the vendored crates and the toolchain.

## Consequences

- **Build-time code is read before it runs,** like the rest of a reviewed crate.
- **Ordinary crates stay usable.** Many safe crates have small build scripts, for example to check the compiler's version.
- **No build step can fetch code.** The sandbox has no network ([0190](0190-cargo-and-build-tool.md)).
- **0005's question on build scripts is answered.**
