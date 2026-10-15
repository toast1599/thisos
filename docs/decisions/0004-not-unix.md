# 0004. Not Unix-like: `target_family` is not `unix`

- **Status:** Accepted
- **Date:** 2026-10-14

## Context

Rust's `target_family` setting decides whether code marked `cfg(unix)` compiles for a target. Declaring `unix` makes many existing crates compile without changes, but those crates then expect the `libc` crate and Unix behavior.

## Options

1. `unix`: compatible with existing crates, but brings Unix assumptions with it.
2. Not `unix`: a clean design, at the cost of porting work.

## Decision

Option 2. The OS is not designed to be Unix-like, and `target_family` is not `unix`. Unix concepts aren't banned. When one is adopted, it gets heavily adapted.

## Consequences

- Crates that only handle `cfg(unix)` or `cfg(windows)` need explicit ports. Examples: getrandom, mio/tokio, socket2.
- The exact `target_family` value (none, or a new one) will be chosen together with the std mapping ([06-system](../design/06-system.md)).
