# 0039. Programs ship as native code, built separately for each architecture

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0008](0008-architecture-neutral-design.md) asks for an architecture-neutral design. One option was a single neutral program format that the device compiles.

## Options

1. **Native code,** built separately for each architecture.
2. **A neutral format** compiled on the device.

## Decision

Option 1.

- Each program is built separately for each architecture it supports.
- The loader runs the native code directly.

## Consequences

- **No compiler on the device.** Running a program doesn't require one.
- **One package per architecture.** A publisher builds and signs a separate package for each architecture ([0040](0040-signed-programs-owner-policy.md), [0041](0041-immutable-packages.md)).
- **Third-party programs may support only some architectures.**
