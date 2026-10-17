# 0200. Contributions come in under the project's license, with no sign-off

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

thisOS is licensed under MIT OR Apache-2.0 ([0199](0199-license.md)). Linux asks contributors to sign off every commit under the Developer Certificate of Origin. Most Rust projects accept contributions under their own license with no sign-off. Some projects have contributors sign an agreement that lets the project relicense.

## Options

1. **The same license, with a sign-off** under the Developer Certificate of Origin.
2. **The same license only.**
3. **A contributor license agreement.**

## Decision

Option 2.

- Contributions are accepted under MIT OR Apache-2.0, the project's own license.
- The README says so, in the wording Rust projects use: unless a contributor states otherwise, what they submit is licensed under both, with no extra terms.
- Commits carry no sign-off, and contributors sign no agreement.

## Consequences

- **Contributing takes no paperwork.**
- **Relicensing would need every contributor's consent,** since nobody assigns rights to the project.
- **Changes to our fork of the `bootloader` crate come in the same way,** and can be offered upstream under the same terms ([0150](0150-bootloader-aarch64.md)).
