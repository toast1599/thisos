# 0275. Eight milestones follow the seven phases

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

The roadmap has 24 steps in 7 phases, each phase ending with a checkpoint ([0227](0227-steps-and-phases.md)). A milestone ends in a working image ([0274](0274-milestone-definition.md)), so none can come before the kernel boots. The first milestone also holds the spikes ([0282](0282-spikes-and-risks.md)) and the boot loader's check ([0283](0283-boot-loader-check-at-step-2.md)).

## Options

1. **Eight, by phase.**
2. **One for each step.**
3. **One for each new capability,** about twelve, across steps.

## Decision

Option 1. The milestones, and the steps each holds:

- **m1:** steps 1 to 3, with the two spikes and the boot loader's check. A kernel boots from a signed image, with its base in place.
- **m2:** steps 4 to 6, the kernel's objects.
- **m3:** steps 7 to 9, user mode on both architectures.
- **m4:** steps 10 to 14, the core services.
- **m5:** steps 15 and 16, multicore and devices.
- **m6:** steps 17 to 20, storage, packages and updates.
- **m7:** steps 21 and 22, users and the network.
- **m8:** steps 23 and 24, third-party programs and the first release.

- Each phase ends in its milestone, in place of its checkpoint. m6 to m8 are phases 6 and 7 with the first release split off.
- This amends 0227, whose phases now end in milestones.

## Consequences

- **The steps keep their order and their content,** apart from the spikes and the boot loader's check ([0282](0282-spikes-and-risks.md), [0283](0283-boot-loader-check-at-step-2.md)).
- **The first release is a milestone of its own,** after the third-party programs it serves.
