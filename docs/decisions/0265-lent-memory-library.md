# 0265. Memory lent by clients goes through one library, measured at checkpoint 4

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

A service keeps each client's state in memory the client lends it, which the client can take back at any time ([0050](0050-clients-pay.md), [0064](0064-memory-belongs-to-payer.md)). Touching it after that is an error, not a crash, through accessors that keep a recovery point in a register ([0065](0065-lost-memory-is-an-error.md), [0133](0133-recovery-point-in-register.md)). So a service can't keep Rust references or collections over a client's state. It copies records in and out through the accessors. The scheduler, the memory service and the default pager keep client state this way too ([0164](0164-root-task-memory-pool.md)), and a fault they don't recover from fails the machine ([0216](0216-essential-service-failure.md)). Generated bindings cover messages ([0142](0142-own-idl.md)), but not a service's own tables. The audit found this cost from the records' text ([0256](0256-audit-findings-as-evidence.md)). Steps 11 to 13 estimate 4,000 to 9,000 lines each in tier 2 ([08-roadmap](../design/08-roadmap.md)).

Exempting the essential services would put their per-client state in the root task's fixed pool ([0164](0164-root-task-memory-pool.md)). A client that can afford many threads could exhaust it, and only a cap would stop that, which [0023](0023-no-quotas.md) rules out.

## Options

1. **One library,** built at step 11 and measured at checkpoint 4.
2. **The essential services exempt.**
3. **Each service on its own.**

## Decision

Option 1.

- Step 11 builds a library for typed, fixed-layout records in lent memory. Each record is read and written only through the accessors ([0133](0133-recovery-point-in-register.md)), and a lost record is an error the service handles.
- The library is in the audited set, beside the accessors ([0191](0191-audited-list-and-wrapper.md)). Every service that keeps per-client state uses it, the scheduler first.
- Checkpoint 4 measures what it costs, in time and in code, under [0264](0264-performance-targets.md)'s targets. A missed target becomes a record under 0201.
- The estimates for steps 11 to 13 are revised at checkpoint 4 ([0249](0249-estimates-and-checkpoints.md)).

## Consequences

- **Services share one tested way to survive lost memory,** instead of each writing its own.
- **0050's model stands,** and its cost becomes known early, in the first service that pays it.
- **The essential services follow the same rule,** so no cap is needed ([0023](0023-no-quotas.md)).
- **Steps 11, 12 and 13 are affected** ([08-roadmap](../design/08-roadmap.md)).
