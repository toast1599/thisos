# 0181. `std::net` uses a network service through the namespace

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Authority comes from capabilities ([0020](0020-capabilities.md)), and programs find services through their namespace ([0131](0131-namespace-path-prefixes.md)). std's sockets assume that any program can reach the network.

## Options

1. **A network service entry.**
2. **A capability for each address or port,** granted by the spawner.

## Decision

Option 1.

- `std::net` uses a network service through the namespace entry `svc/net`.
- Holding that entry is the right to use the network. The service can hand out narrower entries, such as one for outbound connections only.
- Sockets are capabilities to the service, and host names are resolved through it too.

## Consequences

- **A sandbox has no network** unless its spawner passes the entry on.
- **Narrower rights are the service's policy.** It tells them apart by badge ([0128](0128-rights-and-badges.md)).
- **The network stack runs in user mode,** in the network service ([0014](0014-microkernel.md)).
- **06's question on sockets is answered.**
