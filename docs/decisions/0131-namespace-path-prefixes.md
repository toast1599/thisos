# 0131. A namespace maps path prefixes to capabilities

- **Status:** Accepted, amended by [0266](0266-connect-endpoints.md)
- **Date:** 2026-10-17

## Context

A new program receives a namespace: a set of named capabilities to the services it may use ([0028](0028-namespace-at-spawn.md)). 0028 left open its format, and whether a program can pass parts of it on. std's paths have to resolve through capabilities the program holds ([0020](0020-capabilities.md), [06-system](../design/06-system.md)). Fuchsia gives each process a namespace that maps path prefixes to directories.

## Options

1. **Path prefixes,** as in Fuchsia.
2. **Flat names,** with no prefixes matched.

## Decision

Option 1.

- A namespace is a list of entries. Each entry is a UTF-8 path prefix, such as `svc/time` or `data`, with one capability.
- A path resolves through the entry with the longest matching prefix, and the service behind that entry resolves the rest of the path.
- Entries are ordinary capabilities. A program can pass any of them on, under the same name or another.

## Consequences

- **std's paths have a direct mapping** ([06-system](../design/06-system.md)). Opening a path asks the service behind its longest matching prefix.
- **A spawner builds a child's namespace from its own entries,** named as it likes, and only what it passes is reachable ([0028](0028-namespace-at-spawn.md)).
- **Names stay local to each program,** as 0028 says. The same prefix can lead to different services in different programs.
- **Passing an entry on is a copy** ([0123](0123-capabilities-in-messages.md)), which the passer can revoke ([0129](0129-revoke-any-capability.md)).
- **The namespace travels in the startup data** ([0146](0146-startup-data-block.md)).
- **0028's questions on the namespace are answered.**
