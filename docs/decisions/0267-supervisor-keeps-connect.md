# 0267. A restartable service's supervisor keeps its connect endpoint

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

A supervisor restarts a failed service by having its memory taken back ([0105](0105-supervisor-reclaims.md)), which destroys everything made from it ([0053](0053-revoke-destroys-derived.md)), the service's endpoints included. Clients' capabilities to them then fail. If the connect endpoint is the service's too ([0266](0266-connect-endpoints.md)), every namespace entry for the service, in every program already running, points at nothing, and a program can't receive new entries after it starts ([0028](0028-namespace-at-spawn.md)). A restarted filesystem would then be out of reach for every running program. The audit found, by searching the records, that none says what survives a restart ([0256](0256-audit-findings-as-evidence.md)). The essential services are never restarted ([0216](0216-essential-service-failure.md)).

## Options

1. **The supervisor keeps the connect endpoint.**
2. **Each service decides.**
3. **Later work,** with restarts helping only services that have no clients yet.

## Decision

Option 1.

- The supervisor of a restartable service makes the service's connect endpoint from its own memory, and hands each new instance of the service the right to receive on it ([0128](0128-rights-and-badges.md)).
- Namespace entries stay valid across a restart.
- Endpoints made by connects belong to the service's instance, and fail when it's destroyed. The bindings then connect again through the entry ([0266](0266-connect-endpoints.md)).
- Clients rebuild their own state through the new connection, for example by opening their files again.
- This amends 0105, whose supervisor also keeps the connect endpoint of each service it restarts.

## Consequences

- **Restarts work for services that have clients,** as 0014 and 0105 intend.
- **Clients see an error, then a fresh connection,** never a silent change.
- **A service needn't save its clients' state across a restart.** Clients keep what they need to rebuild it.
- **The supervisor pays for one endpoint for each service it restarts** ([0026](0026-programs-pay-kernel-objects.md)).
- **Step 6 is affected, and so is every step that builds a restartable service** ([08-roadmap](../design/08-roadmap.md)).
