# 0086. Priorities stay per program, and each priority is shared by account

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

Fair shares belong to accounts ([0084](0084-shares-belong-to-accounts.md)), while CPU and memory priorities are granted per program ([0033](0033-cpu-fairness.md), [0062](0062-memory-priority-capability.md)). One account can therefore hold programs at different priorities.

## Options

1. **Priorities stay per program.**
2. **Priorities move to accounts.** Every program in an account runs at the account's priorities.

## Decision

Option 1.

- CPU and memory priorities stay per program, as in 0033 and 0062.
- At each priority, the accounts with programs at that priority split the time or memory evenly.
- The programs at that priority in one account split their account's part evenly.

## Consequences

- **One session can mix priorities,** for example a high-priority audio player and a low-priority build in the same account.
- **Strict priorities still come first** ([0033](0033-cpu-fairness.md), [0049](0049-memory-fair-share.md)). Shares apply only among accounts at the same priority.
- **An account can have a part at several priorities,** one for each priority its programs run at.
