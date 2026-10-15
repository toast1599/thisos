# 07. Engineering and freeze

- **Day:** 7 (Tue 2026-10-20)
- **Status:** Not started

## Covers

The toolchain, build system, repository layout, testing, standards, and the freeze itself.

## Decided

- [0018](../decisions/0018-kernel-assurance.md) Kernel assurance: review, tests and fuzzing from the start; formal proofs later
- [0019](../decisions/0019-kernel-minimality.md) Kernel minimality: anything that can reasonably run in user mode does

## Open questions

- **Toolchain:** pinned nightly with `-Zbuild-std`, or our own toolchain fork. Also, how often to upgrade.
- **Build system:** how it enforces the unsafe policy for the kernel and the OS's own services ([0017](../decisions/0017-unsafe-code-kernel-and-services.md)).
- **Enforcing [0008](../decisions/0008-architecture-neutral-design.md):**
  - where architecture-specific code is allowed to live
  - a build check that rejects `cfg(target_arch)` and `core::arch` anywhere else
  - CI that builds and tests every supported architecture
- **Repository layout.**
- **Testing:** a QEMU test harness and CI.
- **Kernel assurance details** ([0018](../decisions/0018-kernel-assurance.md)): review rules, test coverage targets and the fuzzing setup.
- **Enforcing kernel minimality** ([0019](../decisions/0019-kernel-minimality.md)): review rules, a size budget, or both.
- **Coding and documentation standards.**
- **License.**
- **Changes after the freeze:** whether there will be a formal errata process for flaws that implementation turns up.
- **Final review and freeze.**
