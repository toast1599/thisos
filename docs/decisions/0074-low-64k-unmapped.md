# 0074. The lowest 64 KiB of every program's half stays unmapped

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

If low addresses are mapped, null-pointer bugs in unsafe or non-Rust code can reach real memory. Windows and many Linux distributions keep the lowest 64 KiB unmapped. Everything else in a program's half is the loader's choice ([0061](0061-no-randomization-yet.md)).

## Options

1. **The lowest 64 KiB.**
2. **The lowest page only.**
3. **Nothing reserved.**

## Decision

Option 1.

- The loader and the default pager never map a program's lowest 64 KiB.
- The kernel doesn't enforce this.

## Consequences

- **Null-pointer accesses fault** instead of reaching memory. That includes accesses at an offset of up to 64 KiB from null, such as a field of a large struct.
- **It protects only the program itself.** The kernel never reads program memory ([0072](0072-registers-only.md)), so a program that maps its low addresses through a pager of its own harms only itself.
- **Everything else in the lower half is the loader's choice** ([0061](0061-no-randomization-yet.md)).
