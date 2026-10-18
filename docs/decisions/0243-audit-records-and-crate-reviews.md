# 0243. Each audited crate has a record beside the list, and cargo-vet records crate reviews

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

A list in the repository names every crate allowed to contain unsafe code, each linked to its audit record ([0191](0191-audited-list-and-wrapper.md)). 0191 left the records' form to Day 8. Every vendored crate is reviewed before it's used, with a tool such as cargo-vet, and a tool such as cargo-deny refuses crates with known advisories or licenses thisOS can't ship under ([0193](0193-vendored-reviewed-crates.md)). Builds run offline ([0192](0192-build-scripts-reviewed.md)).

## Options

1. **A record for each crate, and cargo-vet.**
2. **`SAFETY` comments only,** with the list naming crates.
3. **One audit log** for every crate.

## Decision

Option 1.

- `audit/` holds the list as a TOML file. It names each crate and its kind, and for a third-party crate its version and hash ([0191](0191-audited-list-and-wrapper.md)).
- Beside the list, each crate has one Markdown record. The record says what the crate's unsafe code is for, gives each unsafe block's safety argument and the invariants it relies on, and names the tests, Miri runs and fuzzing that cover it. It also keeps a log of audits, each with the range it covered, its reviewer and its date.
- A change to unsafe code updates its record in the same change ([0242](0242-review-tiers.md)).
- cargo-vet keeps the reviews of every vendored crate in `supply-chain/`, the build tools' crates included, since the tools produce what's signed. This amends [0193](0193-vendored-reviewed-crates.md), whose reviews also cover the build tools' crates.
- cargo-deny reads a snapshot of the advisory database kept in the repository, since builds run offline ([0192](0192-build-scripts-reviewed.md)).

## Consequences

- **An audit's history stays with its crate.**
- **A reviewer sees what a change does to each safety argument.**
- **0193's examples become the chosen tools.**
- **0191's question on the form of audit records is answered.**
