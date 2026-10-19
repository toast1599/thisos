# 0179. std's `Command` gives a child a copy of the spawner's namespace by default

- **Status:** Accepted, amended by [0268](0268-counter-by-default.md)
- **Date:** 2026-10-17

## Context

Programs start others by explicit spawn ([0022](0022-explicit-spawn.md)), through the loader ([0031](0031-user-mode-loader.md)). Every program has a supervisor, normally its spawner, which learns how it ended ([0104](0104-supervisor.md)) and can have its memory taken back ([0105](0105-supervisor-reclaims.md)). std's `Command` expects a child to inherit its parent's environment, standard streams and current directory, unless told otherwise.

## Options

1. **A copy of the namespace by default.**
2. **An empty namespace by default.**

## Decision

Option 1.

- `Command` calls the loader through the namespace entry `svc/loader`.
- A program's path resolves like any path ([0176](0176-paths-and-current-directory.md)). A bare name is looked up under `/bin`, with no search path.
- By default, a child gets a copy of the spawner's namespace, its environment, its current directory and copies of its standard streams. An extension method narrows the namespace.
- The spawner is the child's supervisor. `Child::wait` waits for the child's exit report or a crash, and `Child::kill` takes back the child's memory ([0105](0105-supervisor-reclaims.md)).
- `process::id` and `Child::id` return a number the loader assigns, unique since boot, which grants nothing. The loader writes it into the program's startup data. This amends 0108, whose list gains the number.

## Consequences

- **0022 holds.** The loader still gives a child only what its spawner passes. Copying the namespace is std's default, carried out by handing capabilities over explicitly, as 0022 expects for the standard streams.
- **Ported programs that run tools work as they do elsewhere.**
- **Confining a child takes one more step.** The spawner narrows its namespace, and grant rights limit what the child can pass on ([0123](0123-capabilities-in-messages.md)).
- **The spawner can revoke what it passed** ([0129](0129-revoke-any-capability.md)).
- **There are no process IDs that grant anything.** The number is only a name ([0020](0020-capabilities.md)).
- **std handles a crashed child itself.** `Child::wait` also watches for the child's suspended threads ([0132](0132-events-notify-then-call.md)), and takes back the child's memory before it returns.
- **0104's question on `Child::wait` is answered.**
