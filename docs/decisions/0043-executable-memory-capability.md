# 0043. Making memory executable at run time needs a capability

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

JIT engines generate code while they run. Under [0014](0014-microkernel.md) they are ordinary programs ([0006](0006-non-rust-software.md)).

## Options

1. **Only with a capability.**
2. **Any program** may make its own memory executable.
3. **Never.** JIT engines would have to interpret instead.

## Decision

Option 1.

- A program can make memory executable only if it holds a capability that allows it.
- Memory is never writable and executable at the same time.

## Consequences

- **A common route for exploits is closed.** Most programs can run only code that came in their signed package ([0040](0040-signed-programs-owner-policy.md)).
- **JIT engines get the capability.** That covers engines such as a WebAssembly runtime or a JavaScript engine.
- **The mechanism is open.** Memory switches between writable and executable through the kernel or the pager ([0027](0027-user-mode-pagers.md)); how is decided on Day 3 ([03-memory](../design/03-memory.md)).
