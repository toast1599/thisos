# 0239. Security problems are reported privately, through GitHub

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The repository is public ([0237](0237-public-repository-and-ci.md)), and no record says how security problems are reported. GitHub offers private vulnerability reporting, and publishes advisories.

## Options

1. **GitHub's private reporting.**
2. **An email address** for reports.
3. **Public issues,** like any other bug.

## Decision

Option 1.

- A `SECURITY.md` in the repository asks reporters to use GitHub's private vulnerability reporting. It's added at step 1.
- A fix is published with an advisory once it's released.

## Consequences

- **Reports stay private until a fix exists.**
- **A flaw found in std's patch also puts the flawed versions on the image's list** of std versions with known flaws ([0210](0210-std-version-declared.md)).
