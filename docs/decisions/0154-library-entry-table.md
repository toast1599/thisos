# 0154. Programs call the system library through a table of entry points

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The system library's functions are the stable ABI between programs and the kernel ([0117](0117-system-library.md)), and programs learn the library's address from their startup data ([0146](0146-startup-data-block.md)). No record said how a call finds each function.

## Options

1. **A table of entry points** at the start of the library.
2. **ELF dynamic symbols,** looked up by name at startup, as programs find Fuchsia's vDSO.
3. **Fixed offsets** for each function.

## Decision

Option 1.

- The library starts with a table of entry points, one for each function, in the order the IDL defines them ([0142](0142-own-idl.md)). Each entry is an offset from the library's start.
- The table only grows ([0143](0143-add-only-versioning.md)), and its length is stored in front of it.
- The library has no writable data and needs no relocations.

## Consequences

- **A call costs one indirect call through the table,** the cost 0117 already counts.
- **std checks the table's length at startup,** so a program built for a newer system fails clearly on an older one.
- **One copy of the library serves every program.** With no writable data and no relocations, its pages are mapped read-and-execute, unchanged, into every address space ([0153](0153-build-unpacks-ramdisk.md)).
- **Programs need no symbol lookup.** std, and the runtimes of other languages ([0006](0006-non-rust-software.md)), find functions by their place in the table.
- **Deprecated functions keep their places** ([0143](0143-add-only-versioning.md)).
