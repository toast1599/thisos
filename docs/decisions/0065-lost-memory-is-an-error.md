# 0065. Touching memory that was taken back is an error, not a crash

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Under [0064](0064-memory-belongs-to-payer.md), a payer can always take memory back, even while another program is using it: memory lent to a service ([0050](0050-clients-pay.md)), and memory shared or handed over ([0058](0058-shared-and-handed-over.md)). Denial of service is in scope ([0016](0016-threat-model-scope.md)).

## Options

1. **An error,** reported to the code that touched the memory.
2. **The program is stopped.**
3. **Revoke waits** until every user lets go.

## Decision

Option 1.

- Touching memory that its payer has taken back is reported as an error to the code that touched it.
- Code that uses memory lent or shared by others goes through accessors built to recover from this, such as generated bindings.

## Consequences

- **A service survives losing a client's memory.** It loses only that client's state and keeps serving everyone else. This answers 03's question on losing lent memory.
- **Taking memory back never waits on its users,** so [0049](0049-memory-fair-share.md) can always reclaim.
- **How the error is delivered is open** ([05-interfaces](../design/05-interfaces.md)). The fault has to reach the code that touched the memory, through its pager or some other way.
- **The accessors need unsafe code.** In the OS's own services, that code belongs to the audited set ([0017](0017-unsafe-code-kernel-and-services.md)).
