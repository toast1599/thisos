# 0231. One repository holds thisOS, with a Cargo workspace for each kind of target

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Cargo builds thisOS in workspaces, driven by a build tool in the repository ([0190](0190-cargo-and-build-tool.md)). 0190 left where each crate lives to Day 8, and so did 0197 for architecture-specific crates ([0197](0197-architecture-check.md)). Cargo builds for one target per invocation, and features unify across a workspace. The kernel builds for the `-none` targets ([0189](0189-kernel-none-targets.md)), programs for the thisOS targets ([0175](0175-rust-target-names.md)), the boot loader for UEFI ([0149](0149-bootloader-crate-uefi.md)), and the tools for the build machine. Vendored crates ([0193](0193-vendored-reviewed-crates.md)), audit records ([0191](0191-audited-list-and-wrapper.md)) and released interface copies ([0196](0196-released-interface-copies.md)) live in the repository.

## Options

1. **One repository, with a workspace for each kind of target.**
2. **Several repositories,** for the kernel, the system's programs, the tools and std's patch.
3. **One workspace,** with each crate's target forced through Cargo's unstable per-package targets.

## Decision

Option 1. The repository is laid out as follows:

```
thisos/
  rust-toolchain.toml  the pinned release (0187)
  tools/           host workspace: the build tool, the compiler wrapper,
                   the IDL compiler, the checkers, the image builder,
                   signing and the test harness
  kernel/          kernel workspace, for the -none targets (0189)
    kernel/          the generic kernel, a library
    kernel-x86_64/   builds the x86_64 kernel
    kernel-aarch64/  builds the AArch64 kernel
    platform-*/      interrupt controllers, timers, UARTs, tables
    sim/             the simulated machine (0233)
  boot/            our fork of the bootloader crate (0150)
  system/          programs workspace, for the thisOS targets
    syslib/          the system library and its architecture crates
    lib/             the page-table library, the IDL runtime, thisos
    services/        the root task, the scheduler, the memory service,
                     the default pager, the loader, ...
    drivers/         the device manager, the PCI bus driver, the IOMMU
                     service, NVMe, xHCI, UARTs, clocks, ...
    tests/           test programs
  std/             std's patch and the target specs (0175, 0187)
  idl/             interface definitions
    released/        frozen copies of released versions (0196)
  vendor/          vendored crates (0193)
  supply-chain/    crate reviews (0243)
  audit/           the list of audited crates, and their records (0191)
  docs/            design, decisions and the roadmap
```

- An architecture- or platform-specific crate sits beside the generic crate it serves, named after it with a suffix, such as `pagetable-x86_64` ([0197](0197-architecture-check.md)).

## Consequences

- **One change can cross the kernel, the system library and the IDL,** as the image does ([0042](0042-updates.md)).
- **Each workspace builds for its own targets,** with core or std built from source ([0187](0187-pinned-stable-toolchain.md), [0189](0189-kernel-none-targets.md)).
- **The build tool knows every workspace,** and runs the checks over all of them ([0190](0190-cargo-and-build-tool.md)).
- **0190's and 0197's questions on where crates live are answered.**
