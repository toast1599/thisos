# 0193. Third-party crates in the OS's own code are vendored and reviewed

- **Status:** Accepted, amended by [0243](0243-audit-records-and-crate-reviews.md), [0285](0285-trusted-publishers.md) and [0287](0287-trusted-crate-updates.md)
- **Date:** 2026-10-18

## Context

The OS's own code may use only crates without unsafe code, or crates whose unsafe parts have joined the audited set ([0017](0017-unsafe-code-kernel-and-services.md)), and the compiler wrapper enforces that ([0191](0191-audited-list-and-wrapper.md)). Safe code can still do harm in a trusted-base service: it can leak or corrupt what the service guards. Build scripts and procedural macros run only in reviewed crates ([0192](0192-build-scripts-reviewed.md)).

## Options

1. **Vendored and reviewed.** Every dependency is vendored and pinned by hash, and each new crate or upgrade is reviewed before use.
2. **Lockfile only.** Cargo's lockfile pins versions and hashes, and safe code isn't reviewed.
3. **No third-party crates** in the OS's own code.

## Decision

Option 1.

- Every crate the OS's own code depends on is vendored in the repository and pinned by hash.
- Each new crate, and each new version of one, is reviewed before it's used. The review is recorded in the repository, with a tool such as cargo-vet.
- A dependency check, with a tool such as cargo-deny, refuses crates with known security advisories, and crates whose licenses can't ship under thisOS's ([0199](0199-license.md)).

## Consequences

- **The build never fetches a crate.** Everything it compiles is in the repository ([0192](0192-build-scripts-reviewed.md)).
- **Upgrading a crate costs a review,** so the OS's own code keeps its dependencies few.
- **Third-party programs choose their own crates,** as normal Rust practice allows ([0017](0017-unsafe-code-kernel-and-services.md)).
