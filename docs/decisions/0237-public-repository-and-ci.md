# 0237. thisOS is public on GitHub, where CI runs the build tool's `ci` command for every change

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Builds are driven by a build tool in the repository ([0190](0190-cargo-and-build-tool.md)), and run offline in a sandbox ([0192](0192-build-scripts-reviewed.md)). On 2026-10-19 the repository had no remote. GitHub's hosted x86_64 runners offer KVM, and GitHub Actions is free without limit on standard runners for public repositories. Its arm64 runners serve public repositories only, and report no nested virtualization. thisOS's license and contribution terms are open ([0199](0199-license.md), [0200](0200-contributions.md)). The project's root key is kept offline ([0205](0205-root-and-working-keys.md)).

## Options

1. **A `ci` command before each merge,** run locally now, and in hosted CI once the repository is published.
2. **Hosted CI from the start,** with the repository published now.
3. **The full suite at checkpoints only.**

For where it's published:

- **Public,** at github.com/toast1599/thisos.
- **Private,** at the same place.

## Decision

Option 2, public.

- The repository is public, at github.com/toast1599/thisos. It's first pushed after Day 8's commit ([0255](0255-freeze.md)).
- The build tool's `ci` command runs every check: builds for both architectures, the build checks, host tests, QEMU tests with x86_64 under KVM and AArch64 emulated, short fuzzing runs, and the reproducibility check ([0241](0241-reproducibility-checks.md)).
- GitHub Actions runs the `ci` command for every change, from step 1 on. Long fuzzing and stress runs are nightly. Timing runs on the laptop ([0240](0240-timing-on-laptop.md)).
- CI holds no signing keys. Test images are signed with test keys ([0205](0205-root-and-working-keys.md)).

## Consequences

- **Anyone can read and rebuild thisOS,** and check its images against their source ([0194](0194-reproducible-builds.md)).
- **The same command runs locally and in CI,** so a local run predicts CI's result ([0238](0238-pushes-and-pull-requests.md)).
- **AArch64 runs emulated in CI,** as it does locally.
- **CI covers both architectures from step 8,** as 0045 requires ([0228](0228-step-start-and-done.md)).
