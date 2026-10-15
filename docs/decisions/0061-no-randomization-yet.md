# 0061. Nothing is randomized at first, and the design keeps randomization possible

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Address randomization places code, data, stacks and heaps at unpredictable addresses, which makes memory-corruption exploits harder. Rust code is largely protected from memory corruption already. The kernel is mapped in every address space ([0047](0047-kernel-mapping.md)), so timing leaks reveal where it is.

## Options

1. **Programs only.** The loader randomizes programs, and the kernel stays at a fixed address.
2. **Programs and kernel.**
3. **Neither.**

## Decision

Option 3 for now.

- Nothing is randomized at first, neither programs nor the kernel.
- The design keeps both possible, so randomization can be turned on later without a redesign.

## Consequences

- **Layouts are reproducible,** which helps early debugging.
- **Exploits against non-Rust programs are easier** until randomization is turned on ([0006](0006-non-rust-software.md)).
- **Nothing may depend on fixed addresses.** No interface exposes kernel addresses to user mode, and nothing outside the loader ([0031](0031-user-mode-loader.md)) assumes where a program's parts are placed.
- **Programs are built position-independent from the start** ([0070](0070-position-independent-programs.md)), so randomizing them later is a change to the loader alone.
- **Randomizing the kernel later** needs it built position-independent, or relocated at boot. It ships in the system image ([0042](0042-updates.md)), so no package has to be rebuilt.
