# 0163. A service hands the first capabilities it issues to the program that started it

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Some rights are issued by user-mode services, not by the kernel, so the root task doesn't receive them at boot ([0024](0024-root-task.md)). The memory service issues the right to grant memory priorities ([0062](0062-memory-priority-capability.md)). The scheduler and the memory service each issue half of the right to make accounts, which goes on to the login service ([0140](0140-accounts-capability-pair.md)).

## Options

1. **Handed to the starter,** in a call at startup.
2. **The starter sets the badges.** It makes the service's endpoint before the service starts, and agrees badge values with it.

## Decision

Option 1.

- A service that issues capabilities hands its first ones to the program that started it, in a call to its supervisor at startup ([0104](0104-supervisor.md)).
- The memory service hands over the right to grant memory priorities and its half of the right to make accounts. The scheduler hands over its half.
- The root task passes copies of the pair of account rights to the login service ([0140](0140-accounts-capability-pair.md)), and memory priorities to the spawners it trusts ([0062](0062-memory-priority-capability.md)).

## Consequences

- **Each service keeps its badge scheme to itself** ([0128](0128-rights-and-badges.md)).
- **Later services work the same way.** Whoever starts a service holds its first capabilities.
- **The root task makes the accounts for the services it starts,** since its own memory is outside the memory service's books ([0164](0164-root-task-memory-pool.md)).
- **What the root task passes on, it can revoke** ([0129](0129-revoke-any-capability.md)).
- **0062's and 0140's questions on how these rights reach the root task are answered.**
