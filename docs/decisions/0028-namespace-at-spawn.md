# 0028. Programs find services through a namespace handed over at spawn

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

With capabilities ([0020](0020-capabilities.md)), there are no global names. Fuchsia hands each program a namespace of service capabilities when it starts.

## Options

1. **A namespace handed over at spawn.**
2. **A directory service.** One capability to a directory service, which the program asks for other services at run time.

## Decision

Option 1.

- When a program is started, its creator hands it a namespace: a set of named capabilities to the services it may use.
- What isn't in its namespace, it can't reach.

## Consequences

- **Names are local to each program.** The same name can lead to different services for different programs.
- **The namespace is part of the startup information** ([0022](0022-explicit-spawn.md), [04-execution](../design/04-execution.md)).
- **std paths resolve through the namespace** ([06-system](../design/06-system.md)).
- **Details come on Day 5:** the namespace's format, and whether a program can pass parts of it on ([05-interfaces](../design/05-interfaces.md)).
