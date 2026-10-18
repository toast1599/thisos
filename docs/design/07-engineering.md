# 07. Toolchain, build system and final review

- **Day:** 7 (Sun 2026-10-18)
- **Status:** Frozen

## Covers

The toolchain, the build system, the license, and the final review of the design. Engineering practices such as modularity and testing moved to Day 8 ([08-roadmap](08-roadmap.md)), and the freeze went with them.

## Decided

- [0018](../decisions/0018-kernel-assurance.md) Kernel assurance: review, tests and fuzzing from the start; formal proofs later
- [0019](../decisions/0019-kernel-minimality.md) Kernel minimality: anything that can reasonably run in user mode does
- [0037](../decisions/0037-kernel-toolchain.md) The kernel is built with one pinned, audited toolchain
- [0060](../decisions/0060-kernel-stack-per-core.md) The kernel has one stack per CPU core, and its worst-case depth is checked at build time

Starting with the toolchain:

- [0187](../decisions/0187-pinned-stable-toolchain.md) The toolchain is a pinned stable Rust release, and std's platform layer is a patch over its library
- [0188](../decisions/0188-upgrade-every-second-release.md) The toolchain is upgraded every second stable release
- [0189](../decisions/0189-kernel-none-targets.md) The kernel builds for the built-in `-none` targets

The build:

- [0190](../decisions/0190-cargo-and-build-tool.md) Cargo builds thisOS, driven by a build tool written in Rust
- [0191](../decisions/0191-audited-list-and-wrapper.md) A list of audited crates, enforced by a compiler wrapper, keeps unsafe code in the audited set
- [0192](../decisions/0192-build-scripts-reviewed.md) Build scripts and procedural macros run only in reviewed crates, and never link native code
- [0193](../decisions/0193-vendored-reviewed-crates.md) Third-party crates in the OS's own code are vendored and reviewed (amended by [0243](../decisions/0243-audit-records-and-crate-reviews.md))
- [0194](../decisions/0194-reproducible-builds.md) The system image and the boot loader build reproducibly
- [0195](../decisions/0195-bindings-safe-output.md) The build tool generates the bindings, and the generated code contains no unsafe code
- [0196](../decisions/0196-released-interface-copies.md) Released interface versions are kept in the repository, and every change is checked against them
- [0197](../decisions/0197-architecture-check.md) Crates are marked by architecture, and the build rejects architecture-specific code in generic crates (amended by [0232](../decisions/0232-traits-and-specific-crates.md))
- [0198](../decisions/0198-stack-depth-checker.md) A checker on the linked kernel bounds its stack depth and rejects recursion

The license, and how changes are made after the freeze:

- [0199](../decisions/0199-license.md) thisOS is licensed under MIT OR Apache-2.0
- [0200](../decisions/0200-contributions.md) Contributions come in under the project's license, with no sign-off
- [0201](../decisions/0201-changes-after-freeze.md) After the freeze, a decision changes only through a new record that cites what building found

## The final review

The review began with the questions left open in [01-objective](01-objective.md) and [02-verification](02-verification.md). After that it went over the whole design, against the threat model ([0016](../decisions/0016-threat-model-scope.md)) and for consistency. Each gap it found became a question that we settled. Its records are also listed in the documents for their topics.

Threat model, signing and packages, which 02 had left open:

- [0202](../decisions/0202-offline-attackers.md) Attackers who get the machine while it's off are in scope
- [0203](../decisions/0203-package-format.md) A package is a signed manifest and files named by SHA-256 Merkle roots
- [0204](../decisions/0204-hybrid-signatures.md) Signatures pair Ed25519 with ML-DSA-65
- [0205](../decisions/0205-root-and-working-keys.md) An offline root key names the working keys, and the boot loader carries only the root
- [0206](../decisions/0206-owner-approves-signers.md) A program runs if the system image's signer or a signer the owner approved signed it
- [0207](../decisions/0207-install-and-start-checks.md) The package store checks packages at install and after each boot, and the loader checks the policy at each start
- [0208](../decisions/0208-boot-loader-beside-old.md) A new boot loader is installed beside the previous one, which the firmware falls back to (amended by [0223](../decisions/0223-qemu-and-laptop-checkpoints.md))
- [0209](../decisions/0209-no-rollback-protection.md) Older signed images still boot, and rollback protection is later work
- [0210](../decisions/0210-std-version-declared.md) Programs declare the std they were built with, and the owner can refuse flawed ones
- [0211](../decisions/0211-relative-relocations.md) Programs carry only relative relocations
- [0212](../decisions/0212-low-image-declaration.md) A program can declare the highest address for its image

The gaps the review found:

- [0213](../decisions/0213-power-capability.md) A power capability lets its holder reboot, power off or fail the machine
- [0214](../decisions/0214-interrupt-remapping.md) Devices raise only the interrupts they were given, through interrupt remapping
- [0215](../decisions/0215-pci-bus-driver-config.md) The PCI bus driver keeps configuration space, and mediates drivers' access to it
- [0216](../decisions/0216-essential-service-failure.md) A failure of an essential service is handled like a fault in the root task
- [0217](../decisions/0217-window-flush-batched.md) Object-window flushes are batched, at most at a set rate
- [0218](../decisions/0218-lock-yield.md) Long kernel operations also yield the kernel lock at their interruption points
- [0219](../decisions/0219-wakeup-preemption.md) A wakeup enters the scheduler only if the woken thread may run before the running one
- [0220](../decisions/0220-endpoint-priority-order.md) Callers waiting on an endpoint are served by priority, then arrival

And slips in earlier records:

- [0221](../decisions/0221-corrections.md) Corrections to 0003 and 0153

Four stale pointers stayed as they were. [0018](../decisions/0018-kernel-assurance.md) and [0019](../decisions/0019-kernel-minimality.md) still say their details come on Day 7, and [0045](../decisions/0045-aarch64-with-first-user-program.md) and [0089](../decisions/0089-interrupts-off-in-kernel.md) point here for topics that moved to Day 8. The last section says where those topics went, and Day 8 settled them: [0242](../decisions/0242-review-tiers.md) and [0245](../decisions/0245-coverage-and-fuzzing.md) for 0018, [0247](../decisions/0247-kernel-size-ratchet.md) for 0019, [0228](../decisions/0228-step-start-and-done.md) and [0237](../decisions/0237-public-repository-and-ci.md) for 0045, and [0246](../decisions/0246-interruption-stretch-limit.md) for 0089.

## Open questions

None are left in this document. The questions that depended on the roadmap ([08-roadmap](08-roadmap.md)) were settled there:

- **Roadmap:** [0227](../decisions/0227-steps-and-phases.md) places the build tool, the checks, the IDL compiler and the compatibility checker. [0242](../decisions/0242-review-tiers.md) and [0243](../decisions/0243-audit-records-and-crate-reviews.md) set the review rules and the form of audit records ([0191](../decisions/0191-audited-list-and-wrapper.md)). [0231](../decisions/0231-repository-layout.md) and [0232](../decisions/0232-traits-and-specific-crates.md) set where architecture-specific crates live, with the repository layout ([0197](../decisions/0197-architecture-check.md)). [0241](../decisions/0241-reproducibility-checks.md) sets how reproducibility is checked ([0194](../decisions/0194-reproducible-builds.md)).
- **Later work:** [0251](../decisions/0251-later-work.md) lists offering the targets to rust-lang/rust ([0187](../decisions/0187-pinned-stable-toolchain.md)), encrypting stored data with keys from the login service and a key bound to the machine ([0202](../decisions/0202-offline-attackers.md)), a signature from Microsoft's UEFI certificate authority for the boot loader ([0205](../decisions/0205-root-and-working-keys.md)), and rollback protection ([0209](../decisions/0209-no-rollback-protection.md)).

## Moved to Day 8

These questions moved to [08-roadmap](08-roadmap.md) along with the engineering practices: where architecture-specific code is allowed to live, CI for every supported architecture, the repository layout, testing, the kernel's assurance details, enforcing kernel minimality, and coding and documentation standards. Day 8 settled them all, in [0228](../decisions/0228-step-start-and-done.md) to [0248](../decisions/0248-coding-and-doc-standards.md).
