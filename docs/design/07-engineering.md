# 07. Toolchain, build system and final review

- **Day:** 7 (Mon 2026-10-19)
- **Status:** Not started

## Covers

The toolchain, the build system, the license, and the final review of the design. The engineering practices, such as modularity and testing, moved to Day 8 ([08-roadmap](08-roadmap.md)), and so did the freeze.

## Decided

- [0018](../decisions/0018-kernel-assurance.md) Kernel assurance: review, tests and fuzzing from the start; formal proofs later
- [0019](../decisions/0019-kernel-minimality.md) Kernel minimality: anything that can reasonably run in user mode does
- [0037](../decisions/0037-kernel-toolchain.md) The kernel is built with one pinned, audited toolchain
- [0060](../decisions/0060-kernel-stack-per-core.md) The kernel has one stack per CPU core, and its worst-case depth is checked at build time

## Open questions

- **Toolchain:** pinned nightly with `-Zbuild-std`, or our own toolchain fork. Also, how often to upgrade. [0037](../decisions/0037-kernel-toolchain.md) requires the kernel's toolchain to be pinned and every upgrade reviewed.
- **Build system:** how it enforces the unsafe policy for the kernel and the OS's own services ([0017](../decisions/0017-unsafe-code-kernel-and-services.md)).
- **Enforcing [0008](../decisions/0008-architecture-neutral-design.md):** a build check that rejects `cfg(target_arch)` and `core::arch` outside architecture-specific code. Under [0046](../decisions/0046-scope-of-0008.md), it covers the OS's own code, platform-specific code included.
- **Kernel stack depth check** ([0060](../decisions/0060-kernel-stack-per-core.md)): which tool checks the kernel's worst-case stack depth at build time, and how it rejects recursion.
- **License.**
- **Changes after the freeze:** whether there will be a formal errata process for flaws that implementation turns up.
- **Final review.**

## Moved to Day 8

These questions moved to [08-roadmap](08-roadmap.md) with the engineering practices: where architecture-specific code is allowed to live, CI for every supported architecture, the repository layout, testing, the kernel's assurance details, enforcing kernel minimality, and coding and documentation standards.
