# 0238. Maintainers push to main after a local `ci` run, and others send pull requests

- **Status:** Accepted, amended by [0291](0291-tags-protected.md)
- **Date:** 2026-10-19

## Context

CI runs the build tool's `ci` command on GitHub for every change ([0237](0237-public-repository-and-ci.md)). Every change is reviewed before it lands, and once there are two maintainers, a tier 1 change needs both ([0242](0242-review-tiers.md)). GitHub doesn't let a pull request's author approve it. The repository has one maintainer, and outside contributors can't push to it.

## Options

1. **Pull requests, with CI required,** for every change.
2. **A local ci run, then a direct push,** with CI on GitHub after.
3. **Pull requests for tiers 1 and 2,** and direct pushes for tier 3.
4. **Direct pushes,** with CI after.

## Decision

Option 2.

- A maintainer reviews a change locally, and pushes it to main once the `ci` command passes locally.
- GitHub runs the same checks after the push. If they fail on main, nothing else is pushed until main passes again, through a fix or a revert. A flaky test counts as a failure, and is fixed, or set aside with a stated reason.
- Contributors who can't push send pull requests. A maintainer reviews each one under the same tiers ([0242](0242-review-tiers.md)), and merges it once CI passes on it.
- A ruleset on GitHub refuses force pushes to main and to the `freeze` tag, and their deletion, for everyone but the repository's owner. The owner force-pushes only by an explicit decision.
- How two maintainers both approve a tier 1 change is settled by a record when a second maintainer joins ([0201](0201-changes-after-freeze.md)).

## Consequences

- **Changes land without waiting for hosted CI,** and main is still checked by a full local run first.
- **A difference between machines shows up after the push,** and is fixed before anything else.
- **Published history isn't rewritten,** except by the owner on purpose.
- **Outside contributions get the same review** as the maintainers' own changes.
