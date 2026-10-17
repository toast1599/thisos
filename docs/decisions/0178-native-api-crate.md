# 0178. The native API is a separate crate

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The native API is the system library together with the bindings generated from the IDL ([0145](0145-std-over-native-api.md)). Programs that need capabilities, regions or notifications use it beside std.

## Options

1. **A separate crate,** with `std::os::thisos` only converting std's objects.
2. **All of it in `std::os::thisos`.**

## Decision

Option 1.

- The native API is a separate crate named `thisos`, generated from the IDL and versioned with the interfaces ([0142](0142-own-idl.md), [0143](0143-add-only-versioning.md)).
- `std::os::thisos` only extends std's own types. It converts std's objects, such as files, sockets, children and threads, to and from the capabilities behind them. It also adds the few settings and details that std's types lack, such as a child's namespace ([0179](0179-command-copies-namespace.md)) or why a program ended ([0180](0180-streams-and-exit-status.md)).

## Consequences

- **The native API evolves without a toolchain release.**
- **std stays small.** Changes to it stay within what upstream would review, once the target is in rust-lang/rust ([0003](0003-rust-std-no-libc.md)).
- **The crate is OS code.** Its unsafe code belongs to the audited set ([0017](0017-unsafe-code-kernel-and-services.md)), like the bindings' ([0142](0142-own-idl.md)).
- **Programs use both together,** for example opening a file with std and passing its capability on with the crate.
