# 0272. The roadmap reopens on what step 1 found, and the rest of the design stays frozen

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

The design and the roadmap froze together with Day 8's commit ([0255](0255-freeze.md)). After the freeze, the roadmap's steps change only through a record that cites what building found, and names the steps it affects ([0250](0250-plan-changes-after-freeze.md)). Step 1 was built on 2026-10-20, and found four things:

- **Its design wasn't done.** Building it took 14 decisions. 11 of them settled topics that no record and no open topic named: the build's sandbox and hosts, the tools' crates, the checks of reviews and advisories, a driver for unsafe code that other crates' macros expand, which reviews to import, test-only crates, stubs for crates nothing builds, workspaces that Cargo won't leave empty, and the advisory and license policies.
- **Its review wasn't sized.** Building cargo-vet and cargo-deny from vendored source meant about 360 crates and 4 million lines. The tools' obvious crates came to 21 crates and 271,000 lines, and imported reviews covered 2 of them at their newest versions.
- **The roadmap can't record progress.** Its step table holds one word for each step. Nothing records what a step designed, added or did, or the evidence for each of 0228's conditions ([0228](0228-step-start-and-done.md)). The words themselves aren't defined.
- **Nothing usable comes before the end.** The roadmap's only tagged result is the first release ([0222](0222-roadmap-to-first-release.md)), and two of its riskiest assumptions are met late: std's patch at step 10 ([0187](0187-pinned-stable-toolchain.md)), and the IOMMU service at step 16 ([0169](0169-iommu-service.md)).

Step 1's work was set aside, uncommitted, so that the roadmap could be redesigned before building resumes.

## Options

1. **Reopen the roadmap alone.**
2. **Reopen the whole design,** and freeze it again with a new tag.
3. **Keep the freeze,** and change the roadmap one record at a time under 0250.

## Decision

Option 1.

- The roadmap reopens: its scope, its steps and phases, its checkpoints, its estimates, its open topics, its later work and its rules for change ([0222](0222-roadmap-to-first-release.md), [0223](0223-qemu-and-laptop-checkpoints.md), [0226](0226-open-topics-at-their-step.md), [0227](0227-steps-and-phases.md), [0228](0228-step-start-and-done.md), [0249](0249-estimates-and-checkpoints.md), [0250](0250-plan-changes-after-freeze.md), [0251](0251-later-work.md)). New records change them, with no further evidence needed.
- The rest of the design stays frozen. A design record changes only under [0201](0201-changes-after-freeze.md), citing what step 1 found, or where a roadmap decision needs one of its lines to stay true.
- The reopening ends when the redesigned roadmap freezes again ([0292](0292-roadmap-refreezes.md)).
- This amends 0250, whose rule waits until then, and 0255, whose freeze of the roadmap ends here.

## Consequences

- **The design's decisions stand.** Only the plan for building them changes, and the records that the plan needs.
- **Step 1's findings shape the plan,** as the audit's checked findings shaped the design ([0256](0256-audit-findings-as-evidence.md)).
- **Building resumes at step 1** once the roadmap freezes again ([0292](0292-roadmap-refreezes.md)).
