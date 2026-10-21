# 0288. The Rust project's releases are trusted, and an upgrade isn't an audit

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

Each toolchain upgrade, about every 12 weeks, audits the changes to unsafe code in `core`, `alloc` and std since the last one, which stay in the audited set ([0188](0188-upgrade-every-second-release.md), [0017](0017-unsafe-code-kernel-and-services.md)). Measured on 2026-10-20, with the pinned release's sources and the nightly of 2026-10-16, about two releases apart:

- 1.99.1's `core`, `alloc` and std hold about 390,000 lines, with `unsafe` 8,331 times.
- Between the two, about 28,500 of those lines changed, and about 1,100 of the changed lines hold `unsafe`.

Third-party crates are trusted by their publishers ([0285](0285-trusted-publishers.md)).

## Options

1. **The Rust project trusted.**
2. **The per-upgrade audit kept,** as 0188 has it.

## Decision

Option 1.

- The Rust project's releases are trusted, like a named publisher ([0285](0285-trusted-publishers.md)).
- Each upgrade passes the full tests on every architecture. It also checks the Rust project's security advisories for the old release and the new one.
- The changes to unsafe code in `core`, `alloc` and std aren't audited.
- std's patch stays our own code, reviewed in tier 1 ([0242](0242-review-tiers.md)).
- `core`, `alloc` and std are on the list as trusted ([0286](0286-unsafe-in-trusted-crates.md)).
- This amends 0188, whose upgrades aren't audits, and 0017, whose `core`, `alloc` and std are trusted rather than audited.

## Consequences

- **An upgrade costs its port and its tests,** not an audit of about 1,100 changed lines.
- **A flaw in the toolchain's library is the Rust project's to fix,** and an upgrade takes the fix ([0188](0188-upgrade-every-second-release.md)).
