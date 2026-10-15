# 0059. Memory that crosses an interface is allocated by the caller, and its use moves only by handover

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Interfaces are defined in an IDL ([0007](0007-abi-calls-and-interfaces.md), Proposed). Freeing memory across an interface, as many C libraries and COM do, is a classic source of leaks and double frees. Clients pay for the state services keep for them ([0050](0050-clients-pay.md)).

## Options

1. **The caller allocates.** The callee uses the caller's buffers only during the call.
2. **The callee allocates.** The caller frees results through the callee's interface.

## Decision

Option 1.

- The caller allocates its buffers, and the callee uses them only during the call.
- Use of memory moves only through an explicit handover ([0058](0058-shared-and-handed-over.md)). Who pays for it never changes ([0064](0064-memory-belongs-to-payer.md)).
- A result whose size isn't known in advance is built in memory the caller provides, then handed back to the caller.
- Every piece of memory has exactly one payer, and nothing is freed across an interface.

## Consequences

- **This matches Rust's ownership model.** Exclusive use moves as ownership does in Rust, so generated bindings can express it in types ([0007](0007-abi-calls-and-interfaces.md), Proposed).
- **It agrees with [0050](0050-clients-pay.md).** A service never pays for a client's results.
- **Results of unknown size need a protocol.** The caller has to provide enough memory, and how it learns how much is open ([05-interfaces](../design/05-interfaces.md)). Bindings can hide this.
- **03's question on memory that crosses interfaces is answered.**
