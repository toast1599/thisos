# 0242. Changes are reviewed in three tiers, by role

- **Status:** Accepted, amended by [0258](0258-outside-input-tier-2.md)
- **Date:** 2026-10-19

## Context

The kernel has strict review rules ([0018](0018-kernel-assurance.md)), and their details were left to the roadmap ([0018](0018-kernel-assurance.md), [0191](0191-audited-list-and-wrapper.md)). A mistake costs more in the kernel, the audited set and the trusted base than elsewhere ([0014](0014-microkernel.md), [0017](0017-unsafe-code-kernel-and-services.md)). Review rules have to hold as people join, so they speak of roles: authors, reviewers and maintainers.

## Options

1. **Three tiers.**
2. **One rule for everything,** at the strictest level.
3. **The kernel strict,** and everything else normal.

## Decision

Option 1.

- **Tier 1** is reviewed line by line against a written checklist. It covers the kernel, our fork of the boot loader, the audited set (std's patch, the system library, the IDL runtime and the accessors), and the build checks that enforce safety: the compiler wrapper, the architecture check, the stack checker and the compatibility checker.
- The tier 1 checklist asks that:
  - the code keeps its invariants;
  - long operations stop at their interruption points, within their work limits ([0246](0246-interruption-stretch-limit.md));
  - nothing recurses ([0198](0198-stack-depth-checker.md)), and no panic can be reached from a system call ([0248](0248-coding-and-doc-standards.md));
  - error paths are tested;
  - each unsafe block has a safety argument;
  - indexes and sizes from programs are masked against speculation ([0248](0248-coding-and-doc-standards.md));
  - architecture code stays in architecture crates ([0232](0232-traits-and-specific-crates.md));
  - the kernel gains no new mechanism without a record ([0247](0247-kernel-size-ratchet.md)).
- A tier 1 change updates its audit record in the same change ([0243](0243-audit-records-and-crate-reviews.md)).
- **Tier 2** gets a full review against the threat model. It covers the trusted-base services: the root task, the scheduler, the memory service, the default pager, the loader, the device manager, the PCI bus driver, the IOMMU service, the filesystem, the package store, the update service and the login service. It also covers the random service, whose output becomes keys ([0174](0174-random-service-and-seed.md)), and the image builder and the signing tools. The review checks for paths to denial of service, who pays for what ([0050](0050-clients-pay.md), [0085](0085-clients-pay-cpu-time.md)), and input from clients.
- **Tier 3** gets a normal review. It covers everything else: other drivers, tools, tests and documents.
- In every tier, CI passes ([0237](0237-public-repository-and-ci.md)), code that follows a decision cites its record ([0248](0248-coding-and-doc-standards.md)), and a frozen decision changes only through a record ([0201](0201-changes-after-freeze.md)).
- A change is never approved by its own author. Once there are two maintainers, a tier 1 change needs both.

## Consequences

- **Review effort goes where a mistake costs most.**
- **The rules hold for any number of people.**
- **A change that touches several tiers gets the strictest review among them.**
- **0018's question on review rules is answered, and so is 0191's.**
