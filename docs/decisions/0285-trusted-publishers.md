# 0285. Third-party crates come from trusted publishers, and their code isn't reviewed

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

Every crate the OS's own code uses is vendored, pinned by hash, and reviewed before it's used ([0193](0193-vendored-reviewed-crates.md)). cargo-vet keeps the reviews, the build tools' crates included ([0243](0243-audit-records-and-crate-reviews.md)), and reviews may be imported from named organizations ([0244](0244-imported-crate-reviews.md)). Step 1 found, on 2026-10-20 ([0272](0272-roadmap-reopened.md)):

- Vendoring cargo-vet and cargo-deny, to run them, meant about 360 crates and 4 million lines, and cargo-vet's own repository exempts 94 of its crates from review.
- The tools' obvious crates came to 21 crates and 271,000 lines. At their newest versions, imported reviews covered 2 of them, and at the best-covered versions about 148,000 lines still had no review.
- Of loom's 30 crates, two had no imported review at any version: loom and generator, about 13,000 lines.

The crates the roadmap expects measured, on the same day:

| Crate | Lines | `unsafe` |
|---|---|---|
| bootloader 0.11.17 | 1,247 | 0 |
| bootloader-x86_64-common 0.11.17 | 2,727 | 49 |
| uefi 0.41.0 | 37,403 | 985 |
| ed25519-dalek 3.0.0 | 3,552 | 4 |
| curve25519-dalek 5.0.0 | 34,781 | 40 |
| ml-dsa 0.1.1 | 3,735 | 0 |
| sha2 0.11.1 | 2,971 | 39 |
| smoltcp 0.14.0 | 55,990 | 23 |
| acpi 6.1.1 | 9,558 | 123 |
| fdt 0.2.0-alpha3 | 7,032 | 12 |
| unwinding 0.2.10 | 3,816 | 178 |
| rand_chacha 0.10.0 | 989 | 0 |

cargo-vet can trust a publisher's releases of a crate, as an entry of its own, beside reviews.

## Options

1. **Trusted crates,** with their vulnerabilities tracked. As for what makes one trusted:
   - **Named publishers.**
   - **Named crates.**
   - **Imported reviews, else named crates.**
2. **Each crate measured, then audited, swapped or replaced by our own.**
3. **Every crate audited,** as 0193 and 0243 have it.

## Decision

Option 1, with named publishers.

- Every third-party crate the build uses comes from a trusted publisher. That covers the image, the tools and the tests.
- A trusted publisher is a crates.io account or a project that the repository names, with its reason, in a reviewed change.
- A crate's version is trusted when a named publisher released it, which cargo-vet's format records as a trusted entry.
- A trusted crate's code isn't reviewed. It's still vendored and pinned by hash, and it still passes the advisory and license checks ([0193](0193-vendored-reviewed-crates.md)).
- A crate from any other publisher can't be used until its publisher is named.
- This amends 0193, whose crates are trusted instead of reviewed, and 0243, whose cargo-vet records trusted publishers instead of reviews. It supersedes 0244, since trust needs no imported reviews.

## Consequences

- **Review goes to our own code.**
- **Trusting a publisher means trusting every release it makes,** which advisories and pinning watch over ([0287](0287-trusted-crate-updates.md)).
- **Vendored code leaves the review estimates** ([0277](0277-sized-work-no-dates.md)).
