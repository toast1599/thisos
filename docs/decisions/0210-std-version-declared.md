# 0210. Programs declare the std they were built with, and the owner can refuse flawed ones

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

A std fix reaches a third-party program only when its publisher rebuilds it ([0042](0042-updates.md)). 0042 left open what the OS does about programs built with a flawed std ([02-verification](../design/02-verification.md)). Each program is isolated, so a flaw harms only that program and what it holds ([0014](0014-microkernel.md)). A program's image declares what it expects ([0147](0147-elf-declarations-only.md)), and the build writes those declarations ([0190](0190-cargo-and-build-tool.md)).

## Options

1. **Declare, warn, and let the owner decide.**
2. **The publisher's job,** with nothing in the OS.
3. **Refuse automatically** programs built with a flawed std.

## Decision

Option 1.

- The build writes into each program's declarations the toolchain release and the version of std's patch it was built with. This amends 0147, whose list of declarations gains them.
- The system image carries a list of std versions with known flaws, which each image update can extend.
- The package store warns the owner about installed programs built with a listed version, and the owner's policy can refuse to run them ([0206](0206-owner-approves-signers.md)).

## Consequences

- **Owners learn which programs need a rebuild,** and decide what to do.
- **A publisher could declare falsely,** but the declarations are signed with the package ([0204](0204-hybrid-signatures.md)), so the publisher answers for them.
- **A program with no such declaration counts as built with an unknown std,** which the owner's policy can treat like a flawed one.
- **0042's question on outdated std in third-party programs is answered.**
