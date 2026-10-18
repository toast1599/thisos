# 0248. The code follows one set of coding and documentation standards

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Day 8 asks for coding and documentation standards ([08-roadmap](../design/08-roadmap.md)). A kernel panic stops the machine ([0110](0110-kernel-panic.md)). Speculative leaks from programs into the kernel are in scope ([0016](0016-threat-model-scope.md)). The audited set carries audit records ([0243](0243-audit-records-and-crate-reviews.md)). The IDL generates the interfaces' reference documentation ([0142](0142-own-idl.md)), and thisOS is licensed under MIT OR Apache-2.0 ([0199](0199-license.md)).

## Options

1. **This set.**
2. **This set, with release builds wrapping on overflow,** Rust's default.
3. **The formatter and default lints,** with `SAFETY` comments on unsafe code.

## Decision

Option 1.

For code:

- Edition 2024, and rustfmt with its default settings. A project set of clippy lints applies, and warnings fail CI.
- Overflow checks are on in every build of the kernel and the OS's own programs, so arithmetic overflow stops instead of wrapping.
- No panic in the kernel can be reached from a system call. `unwrap`, `expect`, `panic!` and unchecked indexing are denied by lints in the kernel's crates, except in boot code, with a stated reason.
- The kernel masks indexes and sizes that come from programs against speculation before it uses them ([0016](0016-threat-model-scope.md)).
- Each unsafe block has a `SAFETY` comment, and each unsafe function a `# Safety` section. Lints enforce both.
- Files carry no license header. Each manifest gives the license as MIT OR Apache-2.0 ([0199](0199-license.md)).

For documentation:

- Each crate opens with what it is, the records it implements, and its invariants.
- Public items are documented, which the `missing_docs` lint enforces.
- Code that follows a decision cites its record.
- The IDL generates the interfaces' reference documentation ([0142](0142-own-idl.md)).
- Records and design documents stay the source of truth for decisions, and [08-roadmap](../design/08-roadmap.md) tracks each step's status.
- Documents and comments use plain English and short sentences, as the design documents do.
- A commit message is a summary sentence in the past tense, then a paragraph, as in the history so far.

## Consequences

- **An overflow stops the program where it happens,** at a small cost in speed.
- **In the kernel, arithmetic on values from programs is checked explicitly,** so its overflow checks catch only the kernel's own bugs.
- **No bad argument can stop the kernel,** since no panic is reachable from a system call.
- **Code leads back to the design** through the records it cites.
- **Day 8's question on coding and documentation standards is answered.**
