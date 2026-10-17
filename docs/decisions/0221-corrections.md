# 0221. Corrections to 0003 and 0153

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The final review on 2026-10-18 found lines in Accepted records that are wrong as written. Records are never edited after the fact, so a later record has to correct them, as [0082](0082-corrections.md) did.

## Options

1. **Amend, with a status note,** as in 0082.
2. **Leave them as history.**

## Decision

Option 1. Each corrected record keeps its decision, and its status gains "amended by 0221". These lines are corrected:

- **[0003](0003-rust-std-no-libc.md), Consequences.** "Building std requires nightly Rust with `-Zbuild-std`": it requires `-Zbuild-std`, a nightly feature, which the toolchain unlocks on a pinned stable release ([0187](0187-pinned-stable-toolchain.md)).
- **[0153](0153-build-unpacks-ramdisk.md), Consequences.** "Both ship in the system image": the image builder doesn't ship. It's a build tool, built from the same source as the kernel ([0190](0190-cargo-and-build-tool.md)), so the header can still change with any image.

## Consequences

- **Both records stay Accepted,** and their status links here.
- **Neither correction is a decision of its own.** Each restates what other records decide.
