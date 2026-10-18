# 0226. Topics no record decides are designed at the step that needs them

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Building every part of the design reaches topics that no record decides ([0222](0222-roadmap-to-first-release.md)). Examples are the filesystem's on-disk format, the log service, the console and the shell, how the login service authenticates users, the protocols between drivers and services, the network stack, where updates come from, and the hardware-discovery service's interface. After the freeze, a frozen decision changes only through a record that cites what building found, and topics the freeze leaves to later work get ordinary records when their time comes ([0201](0201-changes-after-freeze.md)). The kernel's internal layout was left to implementation, with no record ([0073](0073-kernel-layout-in-implementation.md)).

## Options

1. **Designed at their step.** The roadmap lists each open topic with the step that designs it.
2. **Designed now,** before the freeze.
3. **Designed at checkpoints,** in batches between phases.

## Decision

Option 1.

- The roadmap lists each open topic with the step that designs it ([08-roadmap](../design/08-roadmap.md)).
- A step designs its open topics before it writes their code. Their records are ordinary, as for later work ([0201](0201-changes-after-freeze.md)), and need no evidence from building.
- Choices that nothing else depends on, such as internal layouts, algorithms and data structures, need no record, as with [0073](0073-kernel-layout-in-implementation.md).
- An open topic whose design would change a frozen decision follows [0201](0201-changes-after-freeze.md).

## Consequences

- **The freeze covers what's decided,** and says plainly what isn't.
- **Each open topic is designed with what the earlier steps have taught.**
- **The list of open topics is part of the roadmap,** and changes as its steps do ([0250](0250-plan-changes-after-freeze.md)).
