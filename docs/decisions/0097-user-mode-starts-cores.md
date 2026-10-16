# 0097. User mode finds the CPUs, and the root task asks the kernel to start each one

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

Multicore needs the list of CPUs and how to start each one. [0032](0032-minimal-hardware-discovery.md) lets the kernel read only the memory map and information about the interrupt controller and a timer. Under [0060](0060-kernel-stack-per-core.md) and [0068](0068-no-kernel-heap.md), the kernel sets aside a stack for each core at boot.

## Options

1. **The kernel reads it,** alongside the interrupt-controller information, and starts every CPU at boot.
2. **User mode supplies it.** The hardware-discovery service finds the CPUs, and the root task asks the kernel to start each one.

## Decision

Option 2.

- The hardware-discovery service finds the CPUs ([0032](0032-minimal-hardware-discovery.md)).
- The root task asks the kernel to start each one.
- The kernel state for each core it starts, such as the core's stack ([0060](0060-kernel-stack-per-core.md)), is made of kernel objects that are paid for like any other ([0026](0026-programs-pay-kernel-objects.md)), not set aside at boot.
- 0032 stays as written.
- This amends 0068, whose memory set aside at boot no longer includes the stacks of cores started later.

## Consequences

- **The kernel starts on one core** and runs only there until the root task starts more. That core's stack is still set aside at boot.
- **The root task pays for each core it starts.**
- **Which hardware threads share a core reaches the scheduler from user mode** ([0034](0034-smt-trusted-pairs.md), [0093](0093-scheduler-per-core.md)).
- **How the root task names a CPU, and how a CPU is started on each architecture,** come with the boot path ([06-system](../design/06-system.md)). So does what happens if the memory behind a running core is taken back ([0053](0053-revoke-destroys-derived.md)).
