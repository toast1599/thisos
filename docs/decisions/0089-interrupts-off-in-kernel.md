# 0089. The kernel runs with interrupts off between its interruption points

- **Status:** Accepted, amended by [0218](0218-lock-yield.md)
- **Date:** 2026-10-16

## Context

The kernel has one stack per core and never blocks part-way through an operation. Long operations stop at set points where they can be interrupted ([0060](0060-kernel-stack-per-core.md)). Whether interrupts stay off between those points, as in seL4, was left open.

## Options

1. **Off, as in seL4.** Long operations check for pending interrupts at their interruption points.
2. **On, except in short critical sections.**

## Decision

Option 1.

- The kernel runs with interrupts off.
- Long operations check for pending interrupts at their interruption points ([0060](0060-kernel-stack-per-core.md)).

## Consequences

- **Worst-case interrupt latency is the longest stretch between interruption points,** plus entering and leaving the kernel.
- **Kernel code never has to cope with being interrupted,** which keeps review and later proofs simpler ([0018](0018-kernel-assurance.md)).
- **How long a stretch may be** is part of the kernel assurance details ([07-engineering](../design/07-engineering.md)).
- **0060's question is answered.**
