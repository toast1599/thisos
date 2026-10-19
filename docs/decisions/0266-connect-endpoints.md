# 0266. Services spread their callers through a connect endpoint

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Each endpoint has at most one receiving thread, and a service that works on several cores gives each of its threads an endpoint of its own ([0121](0121-callers-help-busy-services.md)). But a namespace entry is one capability ([0131](0131-namespace-path-prefixes.md)), and std's `Command` copies it to children ([0179](0179-command-copies-namespace.md)), so every caller of an entry reaches the same endpoint. All the programs that call `svc/time`, `svc/random` or the loader, on every core, then wait on one thread of the service ([0112](0112-time-service.md), [0174](0174-random-service-and-seed.md)). The audit found this from the records' text ([0256](0256-audit-findings-as-evidence.md)). Interfaces change freely until they're released ([0148](0148-freeze-covers-rules.md)), and the IDL compiler is built at step 6 ([0227](0227-steps-and-phases.md)).

## Options

1. **A connect endpoint,** as a rule.
2. **Each service decides.**
3. **Wait for measurements** ([0264](0264-performance-targets.md)).

## Decision

Option 1.

- A namespace entry for a service names the service's connect endpoint.
- A program's first call there returns an endpoint of its own, which one of the service's threads receives. The service chooses the thread, for example by the caller's core, or by spreading its callers evenly.
- The generated bindings make that first call and keep the endpoint it returns, so neither std nor programs see it.
- A service with one thread may answer every connect with the same endpoint.
- The IDL compiler builds the pattern at step 6.
- This amends 0121, whose services reach their threads' endpoints through connects, and 0131, whose entries for services name connect endpoints.

## Consequences

- **A service can serve callers on several cores at once,** as 0121 intends.
- **Lending still knows which thread to raise,** since each endpoint keeps one receiving thread ([0121](0121-callers-help-busy-services.md)).
- **A connect costs one more call,** once for each program and entry.
- **Revoking an entry stops new connects, not the connections already made,** since the service makes those itself. A spawner that has to cut a program off entirely takes back its memory ([0105](0105-supervisor-reclaims.md)).
- **Step 6 is affected, and so is every step that builds a service** ([08-roadmap](../design/08-roadmap.md)).
