# 0031. The program loader runs in user mode, and starting programs needs a capability

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0019](0019-kernel-minimality.md) moves out of the kernel anything that can reasonably leave it. Loading a program means parsing its executable and building its address space.

## Options

1. **A user-mode loader**, with a capability required to start programs.
2. **A loader in the kernel.**

## Decision

Option 1.

- A user-mode loader parses executables and builds the address spaces of new programs, through capabilities.
- Only a program that holds a capability for creating programs can start others.

## Consequences

- **The kernel never parses executables.** That keeps a large parser that's easy to get wrong out of it.
- **Spawning is a request to the loader.** Explicit spawn ([0022](0022-explicit-spawn.md)) is a request to the loader from a program that holds the capability.
- **The format can change without the kernel.** The executable format ([02-verification](../design/02-verification.md)) is read by the loader, so it can change without changing the kernel.
- **The root task controls who can start programs.** It starts out holding the capability ([0024](0024-root-task.md)) and decides who else gets it.
