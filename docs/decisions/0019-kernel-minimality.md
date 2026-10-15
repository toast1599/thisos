# 0019. Kernel minimality: anything that can reasonably run in user mode does

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0014](0014-microkernel.md) keeps only what has to run in kernel mode in the kernel. Existing microkernels differ in size:
- **seL4**, one of the smallest, keeps address spaces, threads and scheduling, IPC, interrupt delivery and capabilities.
- **Zircon** also keeps memory objects, timers and more kinds of kernel object.

## Options

1. A minimal kernel in the seL4 style.
2. A medium kernel in the Zircon style.

## Decision

The kernel is as small as possible, and this is enforced heavily. Anything that can reasonably be moved to user mode is moved there.

## Consequences

- **Each mechanism is judged on its own.** Which mechanisms stay in the kernel is decided one at a time against this rule ([01-objective](../design/01-objective.md)). seL4's set is the starting point.
- **IPC cost matters even more.** More work moves into user-mode programs, so more operations become messages between programs ([0021](0021-ipc-simple-upgradable.md)).
- **A smaller kernel is easier to check.** It's easier to review, test and eventually prove ([0018](0018-kernel-assurance.md)).
- **Enforcement is still open.** How the rule is kept as the kernel grows (review rules, a size budget, or both) is decided on Day 7 ([07-engineering](../design/07-engineering.md)).
