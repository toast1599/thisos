# 0108. What a new program receives when it's spawned

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

A new program receives exactly the capabilities and arguments its creator hands it ([0022](0022-explicit-spawn.md)), and the loader writes them in ([0107](0107-loader-writes-startup-data.md)). Earlier records require some items, and [04-execution](../design/04-execution.md) asked which others belong.

## Options

Beyond what earlier records require, any of these:

- **Handles to itself:** capabilities to its own address space, capability space and first thread.
- **Standard streams:** capabilities for standard input, output and error.
- **Environment variables:** a list of name=value strings.

## Decision

All three. A new program receives:

- its namespace ([0028](0028-namespace-at-spawn.md));
- its pager ([0081](0081-pager-per-thread.md)), exception handler ([0102](0102-exception-handler.md)) and supervisor ([0104](0104-supervisor.md));
- its CPU and memory priorities ([0033](0033-cpu-fairness.md), [0062](0062-memory-priority-capability.md), [0087](0087-thread-priorities.md)) and its account ([0084](0084-shares-belong-to-accounts.md));
- a core-sharing capability, if its spawner gives it one ([0098](0098-core-sharing-capability.md));
- its arguments ([0022](0022-explicit-spawn.md));
- capabilities to its own address space, capability space and first thread;
- capabilities for its standard input, output and error;
- its environment variables, as a list of name=value strings.

## Consequences

- **A program can make threads and map memory without asking anyone,** through the handles to itself.
- **std's `Command` hands the standard streams over explicitly** instead of letting them be inherited ([0022](0022-explicit-spawn.md)).
- **Environment variables grant nothing.** They're plain strings, not capabilities ([0020](0020-capabilities.md)).
- **How std maps arguments, environment variables and standard streams** comes with the std mapping ([06-system](../design/06-system.md)), and the format with the interfaces ([05-interfaces](../design/05-interfaces.md)).
- **04's question on startup information is answered.**
