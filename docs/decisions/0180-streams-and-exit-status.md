# 0180. Standard streams use a byte-stream protocol, and a program ends with a code or an abnormal end

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

A new program receives capabilities for its standard input, output and error ([0108](0108-startup-information.md)). It reports its own exit status to its supervisor, which is also told when it crashes or is stopped ([0104](0104-supervisor.md)). A message carries up to 64 bytes ([0119](0119-message-eight-words.md)), and bulk data goes through shared regions ([0058](0058-shared-and-handed-over.md)).

## Options

1. **A stream protocol over a region,** and an ending that's either a code or abnormal.
2. **Streams in messages only,** up to 64 bytes per call.

## Decision

Option 1.

- Standard input, output and error are capabilities to a byte-stream protocol defined in the IDL. Data moves through a shared region.
- A missing stream discards writes, and reads as end of file.
- `process::exit` reports its code to the supervisor as an i32.
- A crash or a stop is reported as its own kind of ending. `ExitStatus` shows it as having no code, and the reason is available through an extension ([0178](0178-native-api-crate.md)).

## Consequences

- **Output costs one call per buffer,** not one per line.
- **Pipes, terminals and log services speak one protocol.**
- **Programs started without streams work as usual.**
- **0108's question on how std maps the standard streams is answered,** together with [0179](0179-command-copies-namespace.md).
