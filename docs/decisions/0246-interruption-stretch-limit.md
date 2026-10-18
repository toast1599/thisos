# 0246. Work between interruption points is limited in code, and no stretch may take more than 20 µs

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The kernel runs with interrupts off between its interruption points, so the longest stretch between them is its worst-case interrupt latency ([0089](0089-interrupts-off-in-kernel.md)). At each interruption point, a long operation also yields the kernel lock to a waiting core, so the same stretch bounds how long a core waits for the lock ([0218](0218-lock-yield.md)). Day 8 sets the longest stretch ([0089](0089-interrupts-off-in-kernel.md), [0218](0218-lock-yield.md)). The simulated machine runs the generic kernel on the host ([0233](0233-simulated-machine.md)), and timing runs on the development laptop under KVM ([0240](0240-timing-on-laptop.md)).

## Options

1. **Work limits, and a 20 µs budget.**
2. **Work limits, and a 50 µs budget.**
3. **Work limits, and a 10 µs budget,** which leaves little room for cross-core flushes under KVM.
4. **Work limits only,** with no time budget.

## Decision

Option 1.

- Each long operation does a fixed amount of work between interruption points, such as clearing one 4 KiB page or destroying one object. The limits are constants in the generic kernel.
- The simulated machine counts the work in each stretch. Tests and fuzzing fail when a stretch passes its limit.
- On the laptop under KVM, a test build records the longest stretch, and stress tests fail if any stretch takes more than 20 µs.

## Consequences

- **The kernel's interrupt latency is held to 20 µs on the laptop,** plus entering and leaving the kernel ([0089](0089-interrupts-off-in-kernel.md)).
- **No core waits longer than that for the kernel lock** ([0218](0218-lock-yield.md)).
- **Work limits hold exactly on every architecture.** The time budget is checked on the laptop only, since AArch64 runs emulated.
- **0089's and 0218's question on the longest stretch is answered.**
