# 0282. The first milestone opens with two spikes, and 08-roadmap keeps a risk register

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

Some assumptions the roadmap rests on are met late: that std's platform layer works as a patch over the pinned release's sources (step 10, [0187](0187-pinned-stable-toolchain.md)), that a user-mode IOMMU service is practical (step 16, [0169](0169-iommu-service.md)), that the bootloader crate works on AArch64 (step 7, [0150](0150-bootloader-aarch64.md)), and that the laptop's devices behave ([0223](0223-qemu-and-laptop-checkpoints.md)). If one fails, much of the work before it changes.

## Options

1. **Spikes first, then the order.**
2. **The risky steps moved earlier,** with no spikes.
3. **Each risk met where its step falls,** as now.

For which risks get a spike: std as a patch, the boot loader on AArch64, a user-mode IOMMU, the laptop's devices.

## Decision

Option 1, with two spikes.

- m1 opens with two spikes, beside step 1:
  - **std as a patch:** std is built from the pinned release's sources, patched for a thisOS target spec, with `-Zbuild-std`, against a stub system library;
  - **a user-mode IOMMU:** a small bare-metal program sets up VT-d's tables in QEMU with an emulated IOMMU, from outside the kernel's core, and confines the edu device's DMA.
- A spike is a throwaway prototype. It's never merged, so it's never reviewed. It ends in a written result in the progress log, and in a record wherever the result changes the plan.
- 08-roadmap keeps a risk register: each major risk, where the plan meets it, and what happened. It starts with the two spikes, the boot loader on AArch64 at step 7, and the laptop's devices at each milestone's laptop run.

## Consequences

- **The two riskiest routes are tried before steps depend on them.**
- **A failed spike changes the plan early,** through a record, while that's cheap.
- **The boot loader on AArch64 and the laptop are met where they fall,** and the register tracks them.
