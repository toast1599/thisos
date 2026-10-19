# 0260. Credentials reach the login service only through tier 2 code, and one key combination always reaches it

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Step 21 designs how users authenticate, and the console and terminal ([0226](0226-open-topics-at-their-step.md), [08-roadmap](../design/08-roadmap.md)). The login service decides what each user gets ([0029](0029-users-as-capabilities.md)), and it's reviewed in tier 2 ([0242](0242-review-tiers.md)). No record says how credentials reach it, as the audit found ([0256](0256-audit-findings-as-evidence.md)). Without a rule, they could pass through the console, which carries every program's input and output. A program could also draw a prompt that looks like the login service's. Windows answers that with a key combination that only its login can receive. Attackers who reach a running machine from outside are in scope, and so is what users type ([0257](0257-outside-attackers.md)).

## Options

1. **A requirement now,** with the design at step 21.
2. **A secure attention key, designed now.**
3. **The console trusted,** with no key.
4. **Left to step 21.**

## Decision

Option 1.

- Credentials reach the login service only through tier 2 code: the keyboard drivers ([0258](0258-outside-input-tier-2.md)), and the login service itself. They never pass through the console or any other program.
- One key combination always reaches the login service, and no other program can take it. The login service answers it with its own prompt.
- Step 21 designs how both are met.

## Consequences

- **A fake prompt can be told apart from the real one,** through the key combination.
- **The console never sees a password,** though it's in tier 2 for what users type into programs ([0258](0258-outside-input-tier-2.md)).
- **The login service needs the keyboard and the screen while it asks,** which step 21 designs together with the console.
- **Step 21 is affected** ([08-roadmap](../design/08-roadmap.md)).
