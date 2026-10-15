# 0020. Authority comes from capabilities

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

A program needs some way to be allowed to use a resource. There are two models:
- **Ambient authority**, as in Unix: a program acts with its user's rights over global names.
- **Capabilities**, as in seL4 and Zircon: a program can use only the objects it holds a capability (a handle) to.

## Options

1. Capabilities.
2. Ambient authority.

## Decision

Option 1.

- A program can use only the objects it holds capabilities to.
- Capabilities are passed explicitly.
- Naming something doesn't grant access to it.

## Consequences

- **No ambient authority.** No global name grants access by itself, and no right comes only from who a program runs as.
- **It fits [0015](0015-clean-apis.md).** Capabilities replace integer file descriptors and user IDs as the basis of access.
- **std needs mapping.** Some of std's APIs assume ambient authority, such as opening a file by path. They have to be mapped onto capabilities the program holds ([06-system](../design/06-system.md)). That leans 05's question on the role of std towards std as a layer over the native API ([05-interfaces](../design/05-interfaces.md)).
- **The capability design comes on Day 5:** what rights a capability carries, and how it's passed and revoked ([05-interfaces](../design/05-interfaces.md)).
- **Other mechanisms build on it.** IPC ([0021](0021-ipc-simple-upgradable.md)) and starting programs ([0022](0022-explicit-spawn.md)) are built on capabilities.
