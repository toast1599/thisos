# 0050. Clients pay for the state services keep for them

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Programs pay for their kernel objects with memory they hold ([0026](0026-programs-pay-kernel-objects.md)), so no program can exhaust kernel memory. The same risk exists one level up. A service that keeps per-client state, such as open files, sessions or buffers, in its own memory can be exhausted by a client that keeps opening more. With no quotas ([0023](0023-no-quotas.md)), the service can't cap its clients.

## Options

1. **The client's memory.** The client lends the service memory to hold its state.
2. **The service's memory.** Services allocate client state from their own memory.

For the form of lent memory:

- **Raw memory,** which the service makes into its own pages and objects.
- **The client's pages,** which the client can still map.
- **Either,** chosen by each interface.

## Decision

Option 1, with memory lent in raw form.

- A service keeps per-client state in memory the client lends it.
- The client lends raw memory, which the service makes into its own pages and objects. While it's lent, the client can't read or change them. It can only take the memory back.
- State shared by all clients, such as a cache, stays in the service's own memory.

## Consequences

- **No client can exhaust a service's memory** through per-client state. This closes a route to denial of service ([0016](0016-threat-model-scope.md)) without quotas.
- **Per-client state stays private.** The client can't read it while it's lent, and the kernel clears the memory before the client can reuse it ([0051](0051-kernel-clears-memory.md)).
- **Lent memory counts as the client's,** since the client is its payer ([0064](0064-memory-belongs-to-payer.md)).
- **Lent memory can be taken back at any time,** by the client or under [0049](0049-memory-fair-share.md). The service then gets errors, not a crash, when it touches that client's state, and keeps serving everyone else ([0065](0065-lost-memory-is-an-error.md)).
- **Services get harder to write.** Generated bindings can hide most of the lending ([0007](0007-abi-calls-and-interfaces.md), Proposed).
- **Shared state is the service's own memory,** subject to [0049](0049-memory-fair-share.md) like any other memory.
