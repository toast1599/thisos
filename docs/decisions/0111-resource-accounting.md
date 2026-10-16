# 0111. Each service reports the resource use it already tracks

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

Resource use is measured for visibility, never to cap it ([0023](0023-no-quotas.md)). The scheduler keeps the CPU time books ([0092](0092-scheduler-keeps-time.md)), and the memory service keeps memory accounts ([0025](0025-user-mode-memory.md), [0084](0084-shares-belong-to-accounts.md)).

## Options

1. **Each service reports its own.**
2. **A central accounting service** collects the figures and answers every query.

## Decision

Option 1.

- The scheduler reports CPU time, and the memory service reports memory, per account and per program.
- They report to holders of a capability to read them, such as a system monitor.
- The kernel measures nothing beyond the run times it reports under 0092.

## Consequences

- **Capabilities are covered by memory.** Programs pay for them with memory ([0026](0026-programs-pay-kernel-objects.md)), so memory figures include them.
- **Reading the figures takes a capability.** No program can see others' use by default ([0020](0020-capabilities.md)).
- **No program is added to the trusted path.**
- **04's question on resource accounting is answered.**
