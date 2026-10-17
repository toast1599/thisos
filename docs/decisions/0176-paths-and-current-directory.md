# 0176. Paths that start with `/` resolve through the namespace, and others through a current directory

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

A namespace maps UTF-8 path prefixes, such as `svc/time` and `data`, to capabilities, and a path resolves through its longest matching prefix ([0131](0131-namespace-path-prefixes.md)). There's no global root ([0020](0020-capabilities.md)). std is a layer over the native API, and translates std's paths onto it ([0145](0145-std-over-native-api.md)).

## Options

1. **`/` means the namespace,** and the current directory is a capability.
2. **The namespace only,** with no current directory.
3. **Prefix syntax,** such as `data:/notes.txt`.

## Decision

Option 1.

- Paths are UTF-8, separated by `/`.
- A path that starts with `/` resolves through the namespace's longest matching prefix. The `/` isn't part of the prefix: `/svc/time` resolves through the entry `svc/time`.
- A relative path resolves against the current directory, a directory capability that std keeps. `set_current_dir` changes it.
- A program starts in the directory its spawner names, if any. This amends 0108, whose list gains the starting directory.
- std resolves `..` itself before asking, so a path never climbs above the capability it resolves through.

## Consequences

- **Crates that build paths with `/` work unchanged.**
- **OS strings are always UTF-8 on thisOS.** Every `OsStr` converts to `str` without loss.
- **A directory capability reaches only what's beneath it** ([0020](0020-capabilities.md)). No path can climb out of it.
- **Without a starting directory, relative paths fail** until the program sets one.
- **std's `Command` passes the spawner's current directory on by default** ([0179](0179-command-copies-namespace.md)).
- **06's question on mapping paths is answered.**
