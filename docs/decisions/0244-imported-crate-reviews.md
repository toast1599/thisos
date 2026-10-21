# 0244. Crate reviews may be imported from named organizations, outside the audited set

- **Status:** Superseded by [0285](0285-trusted-publishers.md)
- **Date:** 2026-10-19

## Context

Every vendored crate is reviewed, and the review is recorded in the repository ([0193](0193-vendored-reviewed-crates.md), [0243](0243-audit-records-and-crate-reviews.md)). cargo-vet can import reviews that other organizations publish, such as Mozilla's, Google's and the Bytecode Alliance's. The build's tools need large crates, and later steps need others, such as an ACPI parser, a TCP/IP stack and signature crates ([0227](0227-steps-and-phases.md)). Reviewing all of them ourselves would add hundreds of hours of review. A crate in the audited set always needs an audit record of our own ([0191](0191-audited-list-and-wrapper.md)).

## Options

1. **From named organizations.**
2. **Our own reviews only.**
3. **Imports for the tools' crates only.**

## Decision

Option 1.

- Reviews published by organizations named in the repository count as reviews of crates outside the audited set, at the criteria those reviews state.
- Crates in the audited set get our own audit ([0191](0191-audited-list-and-wrapper.md)), and so do crates that no named organization has reviewed.
- Imported reviews are pinned in the repository, so builds stay offline ([0192](0192-build-scripts-reviewed.md)).

## Consequences

- **Review time goes where it's needed most:** the audited set, and crates that no one else has reviewed.
- **Trusting an import means trusting its publisher,** whom the repository names.
- **0193's rule holds.** Every crate has a review on record before it's used.
