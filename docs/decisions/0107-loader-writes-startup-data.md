# 0107. The loader writes a new program's startup information into its memory

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

A new program receives exactly the capabilities and arguments its creator hands it ([0022](0022-explicit-spawn.md)). The kernel never reads program data, and carries data only in registers ([0072](0072-registers-only.md)), so it can't copy arguments or a namespace into a new program.

## Options

1. **The loader writes it in** before the program's first thread starts.
2. **A region handed over.** The spawner fills a region and hands it over ([0058](0058-shared-and-handed-over.md)) to the new program.
3. **Fetched over IPC.** The new program asks for its startup data after it starts.

## Decision

Option 1.

- The loader writes the arguments, the namespace and the rest of the startup data into the new program's memory before its first thread starts.
- It places the program's capabilities in its capability space.
- The first thread finds the startup data through a register.

## Consequences

- **Nothing passes through the kernel** ([0072](0072-registers-only.md)).
- **The loader already has the access it needs,** since it builds the new program's address space ([0031](0031-user-mode-loader.md), [0052](0052-user-mode-page-tables.md)).
- **A program has its startup data from its first instruction,** with no request to make.
- **The format of the startup data, and how a spawner gives it to the loader,** come with the interfaces ([05-interfaces](../design/05-interfaces.md)).
