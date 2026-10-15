# 0038. Programs ship as ELF, used only as a file container

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

The user-mode loader ([0031](0031-user-mode-loader.md)) reads programs, and every toolchain already produces ELF. [0015](0015-clean-apis.md) rules out adopting a legacy model just for compatibility.

## Options

1. **ELF as a container,** with our own sections.
2. **A format of our own.**

## Decision

Option 1.

- Programs ship as ELF files.
- ELF is used only as a file container. Our own sections carry what the loader needs, such as the capabilities a program asks for.
- None of Unix's process model comes with it.

## Consequences

- **Existing tools work unchanged.** Linkers and inspection tools can produce and inspect programs as they are.
- **The sections' contents come later.** What the custom sections carry is designed with the program model and the namespace ([04-execution](../design/04-execution.md), [05-interfaces](../design/05-interfaces.md)).
- **Signatures and packages need a place in the file,** which is designed with them ([0040](0040-signed-programs-owner-policy.md), [0041](0041-immutable-packages.md)).
