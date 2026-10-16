# 0116. The OS is named thisOS

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

The project had a working title. [01-objective](../design/01-objective.md) deferred the name until the first crates are created. The name also becomes the OS name in the Rust target (`<arch>-unknown-<name>`).

## Options

1. **Name it now.**
2. **Keep the working title** until the first crates are created, as 01 planned.

## Decision

Option 1. The OS is named thisOS.

It's called thisOS because this OS is different from traditional operating systems, and not like the others.

## Consequences

- **The Rust target takes its OS name from it.** Its exact spelling in the target comes with the std mapping ([06-system](../design/06-system.md)).
- **01's question on the project name is answered.**
