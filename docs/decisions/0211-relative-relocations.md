# 0211. Programs carry only relative relocations

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

Programs are built position-independent, and the loader relocates each program as it places it ([0070](0070-position-independent-programs.md)). Every program statically links its own std ([0003](0003-rust-std-no-libc.md)). 0070 left which relocations a program may carry to the program format ([02-verification](../design/02-verification.md)).

## Options

1. **Relative only.** Programs are static position-independent executables.
2. **Symbol relocations too,** so that shared libraries are possible later.

## Decision

Option 1.

- Every program is a static position-independent executable.
- It carries only relative relocations: `R_X86_64_RELATIVE` on x86_64 and `R_AARCH64_RELATIVE` on AArch64, as ordinary entries or packed in the RELR format.
- The loader refuses a program with any other kind of relocation.

## Consequences

- **The loader never looks up a symbol.** Relocating means adding the load address at each listed place.
- **There are no shared libraries.** The system library isn't one: programs call it through its table of entry points ([0154](0154-library-entry-table.md)).
- **Non-Rust programs link statically too** ([0186](0186-non-rust-route.md)). Shared libraries would need a new record.
- **0070's question on relocations is answered.**
