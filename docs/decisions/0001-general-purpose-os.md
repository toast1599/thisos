# 0001. General-purpose production OS

- **Status:** Accepted
- **Date:** 2026-10-14

## Context

The project's purpose sets the bar for everything that follows: the threat model, the compatibility story, and the required level of quality.

## Options

1. A research or teaching system.
2. A specialized system, such as embedded or an appliance.
3. A general-purpose OS built for production.

## Decision

Option 3. This is a general-purpose OS built for production use, not a research or testing system. Getting there may take months or years, depending on outside circumstances. The motivation is to modernize the stack.

## Consequences

- The threat model has to assume programs that don't trust each other: third-party applications, web content, multiple users.
- Software not written in Rust will need a path onto the system eventually ([0006](0006-non-rust-software.md)).
- Rust crates will have to be ported to the OS ([0003](0003-rust-std-no-libc.md), [0005](0005-unsafe-code-policy.md)).
