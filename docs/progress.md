# Progress

This log records, day by day, what was designed, what was added and what was done, with the evidence for each Done ([0279](decisions/0279-progress-log.md)). Every review is recorded here too: each work item reviewed, with its commits, its tier and its reviewer, and for tier 1 the checklist of [0242](decisions/0242-review-tiers.md), item by item. A change isn't reviewed until an entry says so ([0280](decisions/0280-reviews-in-progress-log.md)).

Each working day gets a section, appended at the end, with four parts: **Designed**, **Added**, **Done** and **Next**. Steps and milestones are Not started, In progress or Done ([0281](decisions/0281-step-statuses.md)), and [08-roadmap](design/08-roadmap.md) links to the sections where they change.

## 2026-10-20, Day 9

### Designed

- Step 1 was built, and building it settled 14 decisions. That work was set aside, outside the repository, so that the roadmap could be redesigned first.
- The roadmap reopened on what step 1 found, while the rest of the design stayed frozen ([0272](decisions/0272-roadmap-reopened.md)). Records [0273](decisions/0273-whole-design-in-milestones.md) to [0291](decisions/0291-tags-protected.md) redesigned it:
  - it reaches the whole design through eight milestones, each a working image that's reviewed, passes a written security pass, and is tagged ([0273](decisions/0273-whole-design-in-milestones.md) to [0276](decisions/0276-milestone-security-pass.md));
  - it sizes the work and promises no dates ([0277](decisions/0277-sized-work-no-dates.md));
  - each step opens with a design pass, and its code lands as small work items ([0278](decisions/0278-design-pass-work-items.md));
  - progress and reviews are recorded in this log, and steps take three status words ([0279](decisions/0279-progress-log.md) to [0281](decisions/0281-step-statuses.md));
  - m1 opens with two spikes, and a risk register tracks the plan's risks ([0282](decisions/0282-spikes-and-risks.md));
  - the boot loader checks each image's signature from step 2, and milestone images are development images, for test machines only ([0283](decisions/0283-boot-loader-check-at-step-2.md), [0284](decisions/0284-development-images.md));
  - third-party crates and the Rust project's releases are trusted by their publishers, with their vulnerabilities tracked, instead of reviewed ([0285](decisions/0285-trusted-publishers.md) to [0289](decisions/0289-forked-crates.md));
  - scope can move to later work by record if the measured pace shows the plan far off, and the plan's tags are protected ([0290](decisions/0290-scope-moves-by-record.md), [0291](decisions/0291-tags-protected.md)).
- It froze again with this day's commit, tagged `roadmap-2` ([0292](decisions/0292-roadmap-refreezes.md)).
- Checked on the day, for the records' evidence: the sizes and unsafe code of the crates the plan expects ([0285](decisions/0285-trusted-publishers.md)), and of `core`, `alloc` and std in 1.99.1, with the changes up to the nightly of 2026-10-16 ([0288](decisions/0288-rust-project-trusted.md)).

### Added

- The records [0272](decisions/0272-roadmap-reopened.md) to [0292](decisions/0292-roadmap-refreezes.md), and this log.
- In [08-roadmap](design/08-roadmap.md): the milestones, the spikes, the risk register, the progress section, the re-sized estimates, and the boot loader's check in step 2.
- The status lines of 0017, 0188, 0191, 0193, 0222, 0223, 0226, 0227, 0228, 0238, 0242, 0243, 0249, 0250, 0251, 0255 and 0264, which these records amend, and of 0244, which 0285 supersedes; the decision index; the design documents' notes; the README.

### Done

- The roadmap's redesign, records 0272 to 0292, was accepted. The roadmap freezes with this day's commit, tagged `roadmap-2` ([0292](decisions/0292-roadmap-refreezes.md)).
- The tag ruleset covers `freeze`, `roadmap-2` and the milestone tags `m*` ([0291](decisions/0291-tags-protected.md)).
- No step is done. Step 1 is Not started again, and so are the spikes.

### Next

- Step 1's design pass, starting from the 14 decisions building it settled. Each is put to the architect again only where the redesign changes it: the tools' crates, the checks of reviews and advisories, imported reviews, and test-only crates, which trusted crates now cover ([0285](decisions/0285-trusted-publishers.md) to [0287](decisions/0287-trusted-crate-updates.md)). The others come back as they were.
- The two spikes, beside step 1 ([0282](decisions/0282-spikes-and-risks.md)).
