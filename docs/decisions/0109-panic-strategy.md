# 0109. Third-party programs unwind on panic by default, and the OS's own services abort

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

A Rust program either unwinds or aborts when it panics. Unwinding needs an unwinder, which is a large body of unsafe code, and the OS's own services hold their unsafe code to the audited set ([0017](0017-unsafe-code-kernel-and-services.md)). Aborting pairs with a supervisor that cleans up after the program and may restart it ([0105](0105-supervisor-reclaims.md)).

## Options

1. **Unwind for third-party programs only.**
2. **Unwind everywhere,** the OS's own services included.
3. **Abort everywhere.**

## Decision

Option 1.

- Third-party programs unwind by default, with an unwinder written in Rust.
- The OS's own services abort on panic, and their supervisors decide on a restart ([0105](0105-supervisor-reclaims.md)).

## Consequences

- **`catch_unwind` and Rust's test harness work as usual** in third-party programs.
- **The unwinder stays out of the audited set.** It never runs in the OS's own programs ([0017](0017-unsafe-code-kernel-and-services.md), [0044](0044-system-image-is-os-shipped.md)).
- **No C unwinder is needed** ([0003](0003-rust-std-no-libc.md)).
- **What the kernel does when it panics** is open ([04-execution](../design/04-execution.md)).
