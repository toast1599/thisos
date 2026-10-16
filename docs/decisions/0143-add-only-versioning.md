# 0143. Interfaces only grow, and a tool checks every change

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

[0007](0007-abi-calls-and-interfaces.md) proposed add-only versioning. Programs built against an interface have to keep working as it grows, including third-party programs, which update package by package ([0042](0042-updates.md)). FIDL can also remove methods at new API levels, and services then support a window of recent levels.

## Options

1. **Add-only, checked by a tool.**
2. **Levels with removal,** as in FIDL.

## Decision

Option 1.

- Interfaces only grow. New methods and fields are added at the end.
- Nothing ever changes meaning.
- Every interface carries a version number.
- A tool checks every change against the earlier versions.
- A method can be deprecated, but it's removed only by publishing a new interface under a new name.

## Consequences

- **Old programs never break** on a newer system.
- **Interfaces collect deprecated methods.** A clean break takes a new interface, which services can offer beside the old one.
- **The rules cover every interface defined in the IDL** ([0142](0142-own-idl.md)): the system library ([0117](0117-system-library.md)), the IPC interfaces ([0021](0021-ipc-simple-upgradable.md)) and the startup data ([0146](0146-startup-data-block.md)).
- **The checker runs in the build,** so an incompatible change can't be merged ([07-engineering](../design/07-engineering.md)).
