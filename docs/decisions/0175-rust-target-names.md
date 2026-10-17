# 0175. The Rust targets are `x86_64-unknown-thisos` and `aarch64-unknown-thisos`, with no target family

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The Rust target takes its OS name from the OS's name ([0116](0116-name-thisos.md)). `target_family` decides whether code marked `cfg(unix)` or `cfg(windows)` compiles, and it isn't `unix` ([0004](0004-not-unix.md)). 0004 left open whether it's empty or a new value. Rust target names are lowercase.

## Options

1. **`<arch>-unknown-thisos`, with no family.**
2. **The same names, with a new family, `thisos`.**

## Decision

Option 1.

- The targets are `x86_64-unknown-thisos` and `aarch64-unknown-thisos`.
- `target_os` is `thisos`, `target_vendor` is `unknown`, `target_env` is empty, and there's no `target_family`.

## Consequences

- **Both `cfg(unix)` and `cfg(windows)` are false,** as for UEFI, Hermit and Xous.
- **Code for this OS tests `target_os = "thisos"`.**
- **Crates that handle only Unix and Windows need ports,** as 0004 expected.
- **0004's and 0116's questions are answered.**
