# 0236. Programs write to a second serial port until the log service takes it, and test runs end through the power capability

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The kernel's serial port is the kernel's, and programs can only read its log ([0152](0152-kernel-log-and-reset.md), [0159](0159-root-task-boot-information.md)). User-mode programs, and tests in QEMU, need somewhere to write before a log service exists. The power capability lets its holder reboot, power off or fail the machine ([0213](0213-power-capability.md)). QEMU adds a second serial port for a second serial backend: COM2 on q35, and a second PL011 on virt, as its documentation says (checked on 2026-10-19). The laptop has no serial port ([0223](0223-qemu-and-laptop-checkpoints.md)).

## Options

1. **A second serial port, driven from user mode.**
2. **A kernel log call** behind a capability, as in Zircon's debuglog. It would amend 0152 and 0159.
3. **Pass or fail only,** through the power capability.

## Decision

Option 1.

- The root task drives a second serial port, until the log service takes it over ([0227](0227-steps-and-phases.md)).
- Tests print their results there, in a form the test harness reads.
- A test run ends through the power capability: a power-off when every test passed, and a fail when one didn't ([0213](0213-power-capability.md)).
- On the laptop, the log service shows logs on the framebuffer.
- The kernel is unchanged.

## Consequences

- **The kernel gains no interface** ([0019](0019-kernel-minimality.md)).
- **Kernel output and program output arrive on separate ports,** which the harness reads together.
- **A machine without a second serial port shows program output only once the log service runs.** The kernel shows its own failures on the framebuffer ([0225](0225-framebuffer-log-at-failures.md)).
- **How the root task finds the port before hardware discovery runs is designed at step 8,** and the log service at step 14 ([0226](0226-open-topics-at-their-step.md)).
