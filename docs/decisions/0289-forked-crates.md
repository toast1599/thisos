# 0289. A forked crate's upstream is trusted, and our changes to it are audited

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

We fork the bootloader crate, to add AArch64, the image check and turning off bus mastering ([0149](0149-bootloader-crate-uefi.md), [0150](0150-bootloader-aarch64.md), [0261](0261-bus-mastering-off-at-boot.md)). Tier 1 covers our fork of the boot loader ([0242](0242-review-tiers.md)). Third-party crates are trusted by their publishers ([0285](0285-trusted-publishers.md)).

## Options

1. **Upstream trusted, our changes audited.**
2. **The whole fork audited.**

## Decision

Option 1.

- The version we fork from is trusted, from its publisher, like any crate.
- Every change we make to it is our own code, reviewed in tier 1 when it's in the boot loader or the audited set ([0242](0242-review-tiers.md)).
- The fork's record lists our changes.
- Upstream's fixes merge in like a trusted update ([0287](0287-trusted-crate-updates.md)).
- This amends 0242, whose tier 1 covers our changes to the fork.

## Consequences

- **Our review covers what we wrote.**
- **The fork stays close to upstream,** so that its fixes merge in.
