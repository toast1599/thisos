# 0286. A trusted crate may contain unsafe code, which is recorded but not audited

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

Unsafe code in the OS's own code lives only in one closed, audited set ([0017](0017-unsafe-code-kernel-and-services.md)). A list names each crate allowed to contain it, and a crate joins the list only with an audit record of our own ([0191](0191-audited-list-and-wrapper.md), [0243](0243-audit-records-and-crate-reviews.md)). Most of the crates the roadmap expects contain unsafe code: uefi uses it 985 times, unwinding 178 times, acpi 123 times, curve25519-dalek 40 times and sha2 39 times ([0285](0285-trusted-publishers.md)).

## Options

1. **Allowed, recorded, not audited.**
2. **Only the unsafe parts audited,** and on each update the changed unsafe code.
3. **Only safe crates trusted;** a crate with unsafe code still needs the full audit.

## Decision

Option 1.

- A trusted crate may contain unsafe code.
- It joins the list as trusted, with a short record beside it. The record says what its unsafe code is for, why its publisher is trusted, and its history of advisories.
- Its code isn't audited. The compiler wrapper lets it compile with unsafe code, as it does for the audited set.
- Our own crates keep the full rule: unsafe code only with an audit record of our own ([0017](0017-unsafe-code-kernel-and-services.md), [0243](0243-audit-records-and-crate-reviews.md)).
- The toolchain's library joins the list the same way ([0288](0288-rust-project-trusted.md)).
- This amends 0017, whose closed set also holds trusted crates, 0191, whose list holds them, and 0243, whose records for them are short.

## Consequences

- **The boot loader's crates, the unwinder, the ACPI parser and the signature crates can be used as published.**
- **A flaw in a trusted crate's unsafe code is its publisher's to fix,** and ours to track and take ([0287](0287-trusted-crate-updates.md)).
- **The kernel's own unsafe code stays audited in full.**
