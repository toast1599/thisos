# 0139. A result of unknown size fills what the caller provides, and reports what remains

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

A result whose size isn't known in advance is built in memory the caller provides, then handed back to the caller ([0059](0059-caller-allocates.md)). 0059 left open how the caller learns how much memory to provide.

## Options

1. **Fill, then report.** The service fills as much as fits and says how much more remains.
2. **Ask for the size first,** with a companion call.

## Decision

Option 1.

- The service fills as much of the caller's memory as the result needs, up to what was provided, and reports how much more remains.
- The caller either continues from where the service stopped, or retries with more memory.
- Generated bindings hide the loop ([0142](0142-own-idl.md)).

## Consequences

- **Listings and streams continue in parts,** and results that must arrive whole are retried with more memory. Each interface says which applies ([0142](0142-own-idl.md)).
- **A result can't change size between asking and fetching,** because there's no separate step to ask.
- **Continuing needs a position.** Where the service keeps one for a client, it's per-client state, which the client pays for ([0050](0050-clients-pay.md)).
- **0059's question is answered.**
