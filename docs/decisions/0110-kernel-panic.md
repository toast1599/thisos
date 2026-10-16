# 0110. A kernel panic is logged, then the machine halts or reboots by a boot setting

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

The kernel never unwinds: it builds on `core` alone and has no heap ([0068](0068-no-kernel-heap.md)). Programs' panic strategy is set by [0109](0109-panic-strategy.md), which left open what the kernel does when it panics.

## Options

1. **Log, then halt or reboot** according to a boot setting.
2. **Log and halt.**
3. **Log and reboot.**

## Decision

Option 1.

- When the kernel panics, it stops every core and writes the message to its log output.
- It then halts or reboots, according to a boot setting.

## Consequences

- **Development machines can stay halted for inspection,** and production machines come back on their own.
- **The boot setting comes with the boot path** ([06-system](../design/06-system.md)).
- **04's question on kernel panics is answered.**
