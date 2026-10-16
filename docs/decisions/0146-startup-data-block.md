# 0146. Startup data is a versioned block defined in the IDL

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The loader writes a new program's startup data into its memory and places its capabilities, and the first thread finds the data through a register ([0107](0107-loader-writes-startup-data.md)). [0108](0108-startup-information.md) lists what it contains. 0107 left open the format, and how a spawner gives the data to the loader. Programs now also need the address of the system library ([0117](0117-system-library.md)).

## Options

1. **A block defined in the IDL,** versioned like any interface.
2. **A list of tagged entries,** like ELF's auxiliary vector.

## Decision

Option 1.

- Startup data is a fixed-layout block defined in the IDL ([0142](0142-own-idl.md)), and versioned like any interface ([0143](0143-add-only-versioning.md)).
- Capabilities in it are named by their index in the new program's capability space ([0126](0126-capability-space-radix-table.md)).
- It also gives the address of the system library. This amends 0108, whose list gains that item.
- The spawner fills a region with the arguments, the environment variables and the namespace's names, and hands it over to the loader ([0058](0058-shared-and-handed-over.md)). It passes the capabilities in messages ([0123](0123-capabilities-in-messages.md)).
- The loader checks it all, writes the block into the new program and places the capabilities.

## Consequences

- **Startup data grows by the same rules as every interface** ([0143](0143-add-only-versioning.md)). A program built against an older version still reads it.
- **The loader checks what spawners send,** so a new program never starts from malformed data.
- **A spawner reaches the loader through an interface like any other** ([0031](0031-user-mode-loader.md)).
- **std reads the block at startup** for its arguments, environment and namespace ([06-system](../design/06-system.md)).
- **0107's and 0108's questions on the format are answered.**
