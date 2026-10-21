# 0284. Milestone images run only on test machines, and say they're development images

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

Before the first release, a milestone image lacks defences the design relies on, such as the IOMMU service before m5, updates before m6, and encrypted storage, which is later work ([0202](0202-offline-attackers.md), [0251](0251-later-work.md)). Each milestone ends in an image that boots in QEMU and on the laptop ([0274](0274-milestone-definition.md)).

## Options

1. **Test machines only.**
2. **Use as each defence lands,** with each milestone stating what it's fit for.

## Decision

Option 1.

- Milestone images run only in QEMU, and on the laptop booted from a USB stick. They never hold real data, and are never on a network.
- Each image says it's a development image, on its console at boot and in its version. Step 2 designs the version numbers ([0209](0209-no-rollback-protection.md)).
- The first release is the first image meant for use ([0273](0273-whole-design-in-milestones.md)).

## Consequences

- **No milestone is trusted with what it can't yet protect.**
- **A development image can't be mistaken for a release.**
