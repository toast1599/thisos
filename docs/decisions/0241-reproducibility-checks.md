# 0241. The image and the boot loader are built twice in CI, and rebuilt elsewhere before signing

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The system image and the boot loader build reproducibly ([0194](0194-reproducible-builds.md)), and 0194 left how that's checked to Day 8. The key holder signs what a reproducible build produced ([0205](0205-root-and-working-keys.md)).

## Options

1. **In CI, and before signing.**
2. **Only before releases.**
3. **In CI only.**

## Decision

Option 1.

- Every CI run builds the system image and the boot loader twice, each time with a different directory, user, time, time zone, locale and umask, and compares the bytes.
- Before a release is signed, a second machine rebuilds it from the tagged source, and the hashes must match ([0205](0205-root-and-working-keys.md)).

## Consequences

- **A change that breaks reproducibility fails CI at once.**
- **What's signed is what anyone can rebuild** ([0194](0194-reproducible-builds.md)).
- **0194's question on checking reproducibility is answered.**
