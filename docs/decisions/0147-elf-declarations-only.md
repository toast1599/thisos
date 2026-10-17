# 0147. A program's ELF sections carry only its declarations

- **Status:** Accepted, amended by [0210](0210-std-version-declared.md) and [0212](0212-low-image-declaration.md)
- **Date:** 2026-10-17

## Context

Programs ship as ELF, used only as a file container, with custom sections whose contents were left open ([0038](0038-elf-container.md)). [02-verification](../design/02-verification.md) listed requested capabilities, the signature and package metadata. Installed programs are packages named by their content hash ([0041](0041-immutable-packages.md)), and they're built separately for each architecture ([0039](0039-native-per-arch.md)).

## Options

1. **Declarations only.** The signature and package metadata live in the package around the image.
2. **Everything in the ELF,** as sections.

## Decision

Option 1.

- A program's image declares what it expects: the namespace entries it uses, each with its IDL protocol and version ([0131](0131-namespace-path-prefixes.md), [0142](0142-own-idl.md)), and whether it asks for predictor flushes ([0136](0136-predictor-flush-flag.md)).
- The build writes these declarations from the interfaces the program uses.
- The signature and package metadata live in the package around the image ([0040](0040-signed-programs-owner-policy.md), [0041](0041-immutable-packages.md)).

## Consequences

- **Spawners and the loader can check a program's needs before it starts,** for example that each namespace entry speaks the protocol the program expects.
- **The signature covers the whole image,** with no section left out.
- **Package metadata appears once per package,** not once in each architecture's image ([0039](0039-native-per-arch.md)).
- **The package layout carries the signature and metadata.** It's designed with packages ([02-verification](../design/02-verification.md)).
- **02's question on what the sections carry is answered.** Which relocations a program may carry, and whether its image can be placed at low addresses, stay open.
