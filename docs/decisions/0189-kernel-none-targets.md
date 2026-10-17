# 0189. The kernel builds for the built-in `-none` targets

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The kernel builds on `core` only ([0068](0068-no-kernel-heap.md)). It must not touch floating-point or vector registers: programs' registers are saved on every switch ([0101](0101-eager-fpu-state.md)), and the kernel doesn't save them on entry. Its target spec needs stack probes ([0060](0060-kernel-stack-per-core.md)). Rust already ships bare-metal targets for kernels. Checked with rustc on 2026-10-18:

- `x86_64-unknown-none` turns off SSE and uses soft floating point, has no red zone, uses inline stack probes and the kernel code model, aborts on panic, and builds position-independent.
- `aarch64-unknown-none-softfloat` turns off NEON, requires aligned accesses, uses inline stack probes and aborts on panic.

Both are tier 2 targets upstream.

## Options

1. **The built-in targets.**
2. **Kernel target specs of our own,** such as `x86_64-thisos-kernel`.

## Decision

Option 1.

- The kernel builds for `x86_64-unknown-none` and `aarch64-unknown-none-softfloat`.
- It builds `core` from source with `-Zbuild-std`, with the same compiler and flags as the kernel itself.

## Consequences

- **There's nothing to maintain.** The settings the kernel needs come with the targets.
- **The stack check covers `core`,** which is built with the flag that records stack sizes ([0198](0198-stack-depth-checker.md)).
- **The kernel's target isn't thisOS's.** Programs build for `<arch>-unknown-thisos` ([0175](0175-rust-target-names.md)). The kernel uses no std, so it builds for a bare-metal target.
- **The x86_64 kernel is position-independent already,** which keeps randomizing it later possible ([0061](0061-no-randomization-yet.md)).
