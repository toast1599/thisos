# 0235. Every change runs host, QEMU, check and attack tests, and Miri runs on a test-only nightly

- **Status:** Accepted, amended by [0263](0263-arm64-host-tests.md) and [0264](0264-performance-targets.md)
- **Date:** 2026-10-19

## Context

The kernel has strict review, high test coverage and fuzzing ([0018](0018-kernel-assurance.md)). Speculative leaks and denial of service between programs are in scope, and so are attackers who get the machine while it's off ([0016](0016-threat-model-scope.md), [0202](0202-offline-attackers.md)). One pinned stable release builds everything the project ships, and the tools the build runs ([0187](0187-pinned-stable-toolchain.md)). Miri, which finds undefined behavior in unsafe code, ships only with nightly toolchains. Checked on 2026-10-19 with 1.99.1: AddressSanitizer works on the pinned release with `RUSTC_BOOTSTRAP`. QEMU has edu, a PCI device that does DMA and MSI.

## Options

1. **The full set, with Miri on a test-only nightly.**
2. **The full set, without Miri.**
3. **The full set, without attack tests.**

## Decision

Option 1. The build has these tests:

- **Host, on every change:** unit tests for every generic crate, the generic kernel on the simulated machine included ([0233](0233-simulated-machine.md)); and Miri and AddressSanitizer over the audited set's code that runs on the host.
- **QEMU, on every change:** the kernel's tests of architecture and platform code, user-mode test programs, upstream's std tests by stage ([0229](0229-std-stages.md)), and tests that boot whole images, with and without an IOMMU.
- **Checks:** each build check, such as the compiler wrapper, the architecture check, the stack checker and the compatibility checker, is fed planted violations, which it must reject ([0191](0191-audited-list-and-wrapper.md), [0196](0196-released-interface-copies.md), [0197](0197-architecture-check.md), [0198](0198-stack-depth-checker.md)).
- **Attacks:** each defence the design records against an in-scope threat has a test that tries the attack. Examples are window-flush loops ([0217](0217-window-flush-batched.md)), revokes that would hold the lock ([0218](0218-lock-yield.md)), wakeup storms ([0219](0219-wakeup-preemption.md)), flooding clients ([0050](0050-clients-pay.md), [0085](0085-clients-pay-cpu-time.md)), inversion through a busy service ([0121](0121-callers-help-busy-services.md)), a device writing outside its mappings ([0169](0169-iommu-service.md), [0214](0214-interrupt-remapping.md)), and tampered images and packages ([0156](0156-boot-loader-checks-image.md), [0203](0203-package-format.md)).
- **Regressions:** every bug fix adds a test that failed before it.
- **Benchmarks:** IPC round trips, faults and switches, tracked over time and not gated ([0240](0240-timing-on-laptop.md)).
- Miri runs on a pinned nightly. It builds nothing that ships, and no tool the build runs, so [0187](0187-pinned-stable-toolchain.md) holds. It moves with each toolchain upgrade ([0188](0188-upgrade-every-second-release.md)).

## Consequences

- **Defences are tested as attacks,** not only as features.
- **Unsafe code is checked for undefined behavior** wherever it runs on the host.
- **Each toolchain upgrade picks a nightly to match the new release** ([0188](0188-upgrade-every-second-release.md)).
- **Day 8's question on testing is answered,** together with [0236](0236-program-output-second-serial.md) to [0241](0241-reproducibility-checks.md).
