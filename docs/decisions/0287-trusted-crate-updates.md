# 0287. Trusted crates stay pinned, and move when an advisory or a fix calls for it

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

Each new version of a vendored crate is reviewed before it's used, and a dependency check refuses crates with known advisories ([0193](0193-vendored-reviewed-crates.md)). Trusted crates aren't reviewed ([0285](0285-trusted-publishers.md)).

## Options

1. **Pinned, and moved on an advisory or a needed fix.**
2. **The same, with each update's changes to unsafe code reviewed.**
3. **Moved to the newest version at each toolchain upgrade.**

## Decision

Option 1.

- Trusted crates stay pinned in the lockfiles.
- The advisory check runs on every change, and nightly against RustSec's live database.
- A crate moves to a new version when an advisory applies or a fix is needed. The change that moves it says why, and summarizes the changelog. Its code isn't reviewed again.
- This amends 0193, whose new versions aren't reviewed.

## Consequences

- **A known flaw shows up within a night,** and its fix is taken on purpose.
- **Nothing moves without a reason on record.**
