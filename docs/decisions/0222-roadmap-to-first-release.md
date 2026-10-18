# 0222. The roadmap runs from an empty repository to a first release of the whole design

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Day 8 asks for the exact plan for building thisOS: the steps from an empty repository to a working system, with a place for every part of the design ([08-roadmap](../design/08-roadmap.md)). The README's rough plan ran to "Month 3+: SMP, networking", with WebAssembly later ([README](../../README.md)). Some decided work is already left for later, such as non-Rust software ([0186](0186-non-rust-route.md)), swap ([0077](0077-swap-in-default-pager.md)), randomization ([0061](0061-no-randomization-yet.md)), encrypted storage ([0202](0202-offline-attackers.md)) and rollback protection ([0209](0209-no-rollback-protection.md)).

## Options

1. **The whole design, to a first release.** The roadmap ends when every Accepted decision not on the list of later work is built and tested, and a first release is published.
2. **A usable system, then a second plan.** It ends at a system that boots to a shell with storage and std programs. The rest is planned at that point.
3. **The kernel and the core services.** It ends when the root task, the scheduler, the memory service, the default pager and the loader run on both architectures.

## Decision

Option 1.

- The roadmap ends when every Accepted decision that isn't on the list of later work ([0251](0251-later-work.md)) is built, and tested on both architectures.
- It ends with a first release: a signed system image and boot loader, the SDK that third-party programs build with, and version 1 of the interfaces they reach ([0230](0230-interfaces-released-with-sdk.md)).

## Consequences

- **Every part of the design has a step** ([0227](0227-steps-and-phases.md)). Ordering problems show up now, while the design can still change at little cost.
- **Later steps are estimated less precisely.** Checkpoints re-estimate them as the work goes ([0249](0249-estimates-and-checkpoints.md)).
- **The README's rough plan is replaced** by the steps in [08-roadmap](../design/08-roadmap.md).
- **Day 8's question on the steps is answered,** together with [0227](0227-steps-and-phases.md) and [0228](0228-step-start-and-done.md).
