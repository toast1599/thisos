# 0196. Released interface versions are kept in the repository, and every change is checked against them

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

Interfaces only grow, and a tool checks every change against the earlier versions ([0143](0143-add-only-versioning.md)). The checker runs in the build, so that an incompatible change can't be merged ([0143](0143-add-only-versioning.md)). Each interface's version 1 is fixed when it's first released ([0148](0148-freeze-covers-rules.md)).

## Options

1. **Released copies in the repository.** Each released version is kept as a frozen copy, and the checker compares against every copy.
2. **Release tags.** The checker compares against the definitions at each release tag in the repository's history.

## Decision

Option 1.

- Every released version of every interface is kept in the repository, as a frozen copy of its definition.
- Releasing a version ([0148](0148-freeze-covers-rules.md)) means adding its copy, in a reviewed change. A copy never changes afterwards.
- On every change, the checker compares the current definitions against every released copy. A failed check blocks the merge.

## Consequences

- **A release is a visible change:** the one that adds the copy. The roadmap's releases ([0148](0148-freeze-covers-rules.md)) become concrete steps.
- **The baselines don't depend on tags or history,** so they can't be lost or moved.
- **Unreleased interfaces change freely,** as 0148 allows, because nothing is compared until a copy exists.
- **05's question on the checker in the build is answered.**
