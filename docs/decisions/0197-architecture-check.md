# 0197. Crates are marked by architecture, and the build rejects architecture-specific code in generic crates

- **Status:** Accepted, amended by [0232](0232-traits-and-specific-crates.md)
- **Date:** 2026-10-18

## Context

Nothing except architecture-specific code relies on any architecture-specific concept ([0008](0008-architecture-neutral-design.md)). That applies to the OS's own code, and covers platform-specific code too ([0046](0046-scope-of-0008.md)). 07 asked for a build check that rejects `cfg(target_arch)` and `core::arch` outside architecture-specific code. Where that code lives is part of Day 8's question on modularity ([08-roadmap](../design/08-roadmap.md)).

## Options

1. **Crate marks and a build check.** Each crate is marked as generic, or as specific to an architecture or platform, and the build rejects architecture-specific constructs in generic crates.
2. **Module marks and a source scan.** Architecture code may sit in marked modules of generic crates, and a parser-based scanner rejects the same constructs elsewhere.
3. **Review only.**

## Decision

Option 1.

- Each crate of the OS's own code is marked in its manifest as generic, or as specific to one architecture or one platform.
- In a generic crate, the build rejects `cfg(target_arch)`, `cfg(target_feature)`, `#[target_feature]`, paths into `core::arch` and `std::arch`, and the `asm!`, `global_asm!` and `naked_asm!` macros.
- The check reads each crate's source before `cfg` is applied, and its resolved paths after, so a macro or a re-export can't hide a use.

## Consequences

- **Architecture-specific code lives in crates of its own.** Which crates, and where, is Day 8's ([08-roadmap](../design/08-roadmap.md)).
- **The build can't spot platform-specific code.** An interrupt-controller driver uses no special syntax, so keeping platform code in platform crates stays a review rule ([0046](0046-scope-of-0008.md)).
- **std is checked by review only.** It's built from upstream's sources, which mix architectures inside each crate, and our patch follows upstream's layout ([0187](0187-pinned-stable-toolchain.md)).
- **The 64-bit guarantees aren't architecture-specific** ([0010](0010-64-bit-only.md)), so generic code may still rely on 64-bit pointers and atomics.
- **07's question on enforcing 0008 is answered.**
