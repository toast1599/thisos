# 0291. The tag ruleset protects the freeze, roadmap-2 and the milestone tags

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

A ruleset refuses force pushes to main and to the `freeze` tag, and their deletion, for everyone but the repository's owner ([0238](0238-pushes-and-pull-requests.md)). The redesigned roadmap freezes with a tag of its own ([0292](0292-roadmap-refreezes.md)), and each milestone is tagged ([0274](0274-milestone-definition.md)).

## Options

1. **Every one of them protected.**
2. **Only roadmap-2,** beside freeze.
3. **The ruleset as it is.**

## Decision

Option 1.

- The tag ruleset covers `freeze`, `roadmap-2` and the milestone tags, `m*`.
- It refuses their deletion, their updates and force pushes, for everyone but the repository's admin role.
- This amends 0238, whose ruleset covers these tags.

## Consequences

- **A tag that marks a frozen plan or a finished milestone stays where it was put.**
