# 07. Engineering and freeze

- **Day:** 7 (Tue 2026-10-20)
- **Status:** Not started

## Covers

The toolchain, build system, repository layout, testing, standards, and the freeze itself.

## Decided

- [0018](../decisions/0018-kernel-assurance.md) Kernel assurance: review, tests and fuzzing from the start; formal proofs later
- [0019](../decisions/0019-kernel-minimality.md) Kernel minimality: anything that can reasonably run in user mode does
- [0037](../decisions/0037-kernel-toolchain.md) The kernel is built with one pinned, audited toolchain
- [0060](../decisions/0060-kernel-stack-per-core.md) The kernel has one stack per CPU core, and its worst-case depth is checked at build time

## Open questions

- **Toolchain:** pinned nightly with `-Zbuild-std`, or our own toolchain fork. Also, how often to upgrade. [0037](../decisions/0037-kernel-toolchain.md) requires the kernel's toolchain to be pinned and every upgrade reviewed.
- **Build system:** how it enforces the unsafe policy for the kernel and the OS's own services ([0017](../decisions/0017-unsafe-code-kernel-and-services.md)).
- **Enforcing [0008](../decisions/0008-architecture-neutral-design.md):**
  - where architecture-specific code is allowed to live
  - a build check that rejects `cfg(target_arch)` and `core::arch` anywhere else
  - CI that builds and tests every supported architecture, x86_64 and AArch64 from the first user-mode program on [0045](../decisions/0045-aarch64-with-first-user-program.md)
  - under [0046](../decisions/0046-scope-of-0008.md), these checks cover the OS's own code, platform-specific code included
- **Repository layout.**
- **Testing:** a QEMU test harness and CI.
- **Kernel assurance details** ([0018](../decisions/0018-kernel-assurance.md)): review rules, test coverage targets and the fuzzing setup.
- **Kernel stack depth check** ([0060](../decisions/0060-kernel-stack-per-core.md)): which tool checks the kernel's worst-case stack depth at build time, and how it rejects recursion.
- **Enforcing kernel minimality** ([0019](../decisions/0019-kernel-minimality.md)): review rules, a size budget, or both.
- **Coding and documentation standards.**
- **License.**
- **Changes after the freeze:** whether there will be a formal errata process for flaws that implementation turns up.
- **Final review and freeze.**
