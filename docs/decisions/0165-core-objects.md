# 0165. Each core is a kernel object, named by its capability

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

User mode finds the CPUs, and the root task asks the kernel to start each one ([0097](0097-user-mode-starts-cores.md)). Each core's kernel state is made of kernel objects that the root task pays for. The scheduler has a thread on each core ([0093](0093-scheduler-per-core.md)), and binds threads to cores ([0134](0134-scheduler-interface.md)).

## Options

1. **Core objects,** named by their capabilities.
2. **Numbers the kernel assigns,** in the order cores are started.

## Decision

Option 1.

- Each core is a kernel object that the root task makes from its own memory, with the core-start capability it receives at boot ([0159](0159-root-task-boot-information.md)). The core is named by its capability.
- The root task makes it from the CPU's hardware ID in the firmware tables, the APIC ID on x86_64 or the MPIDR on AArch64. On AArch64, it also gives the start method: PSCI, through SMC or HVC.
- The kernel starts the core with startup IPIs on x86_64, through a low page set aside at boot, and with PSCI's CPU_ON on AArch64.
- The boot core's capability is among the root task's initial capabilities ([0159](0159-root-task-boot-information.md)).
- The scheduler binds threads to cores through copies of these capabilities, together with the scheduling-control capability ([0134](0134-scheduler-interface.md)). It names its thread for a core before the core starts ([0161](0161-root-task-before-scheduler.md)).

## Consequences

- **There are no global core numbers.** Binding a thread to a core takes the core's capability ([0020](0020-capabilities.md)).
- **A new core runs its scheduler thread first** ([0162](0162-scheduler-above-priorities.md)).
- **The low page holds start-up code,** set aside at boot with the kernel's code ([0068](0068-no-kernel-heap.md)). On x86_64, other cores start in real mode, below 1 MiB.
- **The hardware-discovery service reports each CPU's hardware ID and start method** to the root task ([0032](0032-minimal-hardware-discovery.md)), and reports to the scheduler which hardware threads share a core ([0034](0034-smt-trusted-pairs.md)).
- **0097's questions on naming and starting a CPU are answered.**
