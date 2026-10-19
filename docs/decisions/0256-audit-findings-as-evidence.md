# 0256. Checked findings of the audit after the freeze count as evidence under 0201

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The design and the roadmap froze with Day 8's commit ([0255](0255-freeze.md)). After the freeze, a frozen decision changes only through a new record that cites what building found: a failing test, a measurement or a bug ([0201](0201-changes-after-freeze.md)). Its purpose is that opinion alone doesn't reopen the design. The roadmap's steps change the same way, and its estimates change only at checkpoints ([0250](0250-plan-changes-after-freeze.md), [0249](0249-estimates-and-checkpoints.md)).

On 2026-10-19, before step 1 began, the whole design was audited. Nothing is built yet, so none of its findings can come from building. Several of them rest on checks, not on opinion:

- The library sources of the pinned release, 1.99.1, and of the nightly of 2026-10-16 have queue-based versions of std's `RwLock` and `Once`, but none of its `Mutex` or `Condvar` ([0177](0177-locks-on-parking.md)).
- QEMU's design lets an emulated guest see stronger memory ordering than its architecture's, never weaker. AArch64 emulated on an x86_64 host sees close to x86's ordering ([0223](0223-qemu-and-laptop-checkpoints.md), [0227](0227-steps-and-phases.md)).
- No record puts attackers who reach a running machine from outside in scope ([0016](0016-threat-model-scope.md), [0202](0202-offline-attackers.md)), and tier 2 leaves out the network service, the USB stack and the console ([0242](0242-review-tiers.md)).
- No record covers DMA before the IOMMU service runs, or the source check on remapped interrupts ([0169](0169-iommu-service.md), [0214](0214-interrupt-remapping.md)).
- The roadmap's review hours add up to 1,240, not the 1,230 it states. Vendored code that needs our own audit ([0244](0244-imported-crate-reviews.md)) is costed at 262 and 412 lines an hour, against tier 1's 150 ([08-roadmap](../design/08-roadmap.md)).
- Published work shows that a thread counting in shared memory makes a precise timer (Schwarz, Maurice, Gruss and Mangard, 2017), and browsers had to restrict shared memory itself, not only their clocks ([0113](0113-counter-capability.md)).

## Options

1. **Checked findings count as evidence.** One record admits them, and each record that follows cites it and the check behind its finding.
2. **Open topics and tests only.** No frozen record changes. Findings that fit an open topic or a step's tests go there, and the rest wait for building to show them.
3. **Reopen as a design day,** amend freely, then freeze again with a new tag.

## Decision

Option 1.

- A finding of the audit of 2026-10-19 counts as evidence under 0201 when a check confirms it: the sources of a tool or a library, a tool's documentation, published work, the text of the records, or the roadmap's own arithmetic.
- Each record that follows from the audit cites this record and the check behind its finding, and names the steps it affects, as 0201 requires.
- A finding that no check confirms is opinion, and still waits for building.
- The roadmap's estimates are corrected now, before step 1, since their errors are in arithmetic, not in guesses that measuring would replace. From checkpoint 1 on, they change only at checkpoints again ([0249](0249-estimates-and-checkpoints.md)).
- The `freeze` tag stays where it is.
- This amends 0201, whose evidence also includes these findings, and 0250, whose estimates are also corrected this once, before any checkpoint.

## Consequences

- **The freeze still holds.** Only checked findings get through, as only evidence from building did before.
- **Known gaps are closed before code is built on them,** while a change still costs one record.
- **The audit's records are listed with the others written after the freeze** ([0201](0201-changes-after-freeze.md)).
- **A later audit would need a record of its own.** This one covers only the audit of 2026-10-19.
