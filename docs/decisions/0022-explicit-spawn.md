# 0022. Programs start other programs by explicit spawn

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0015](0015-clean-apis.md) rules out copying `fork` and `execve`. Fuchsia shows an alternative: the creator builds the new process and hands it the handles it needs.

## Options

1. Explicit spawn: the creator hands the new program exactly the capabilities and arguments it needs.
2. Spawn with defaults: the new program inherits a default set unless told otherwise, like `posix_spawn`.

## Decision

Option 1.

- A program starts another by creating it and handing it exactly the capabilities and arguments it needs.
- Nothing is inherited by default.

## Consequences

- **A new program starts with nothing else.** It can't reach anything its creator didn't hand it ([0020](0020-capabilities.md)).
- **std's `Command` fits.** It already works by spawning. Some of its defaults involve inheriting, such as standard input and output. std's platform layer carries those out by handing the capabilities over explicitly ([0015](0015-clean-apis.md)).
- **Startup information is designed later.** Which capabilities and startup information a new program receives, and in what form, is designed with the program model ([04-execution](../design/04-execution.md)) and the executable format ([02-verification](../design/02-verification.md)).
