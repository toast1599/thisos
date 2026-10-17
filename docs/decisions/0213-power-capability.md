# 0213. A power capability lets its holder reboot, power off or fail the machine

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

No record covers shutting down or rebooting normally. On AArch64, power-off and reset go through PSCI, which user mode can't call. On x86_64, power-off writes sleep values that come from ACPI's bytecode, which the kernel doesn't parse ([0032](0032-minimal-hardware-discovery.md)). The kernel already resets the machine after a panic ([0152](0152-kernel-log-and-reset.md)). After a kernel panic or a fault in the root task, the machine halts or reboots by a boot setting ([0110](0110-kernel-panic.md), [0115](0115-root-task-fault-boot-setting.md)).

## Options

1. **A power capability** that asks the kernel.
2. **User mode where possible.** On x86_64, a power service does it all through I/O ports and device memory. The kernel offers PSCI calls on AArch64 only.
3. **Later work.**

## Decision

Option 1.

- The root task receives a power capability at boot. This amends 0159, whose list of initial capabilities gains it.
- With it, a program can ask the kernel to:
  - reboot, by the method the kernel already uses after a panic ([0152](0152-kernel-log-and-reset.md));
  - power off, through PSCI on AArch64, or on x86_64 by writing the sleep values the caller passes, which the hardware-discovery service read from the firmware's tables;
  - fail: log a message, then halt or reboot by the boot setting, as after a kernel panic ([0110](0110-kernel-panic.md)).
- A shutdown is a user-mode sequence: programs are stopped first, then the capability is used.
- On x86_64, the kernel reads where the power-off registers are from ACPI's FADT, which it already reads for the reset. This amends 0032, whose list of what the kernel reads gains them, and 0046, whose limit on the kernel's platform code gains the power-off.

## Consequences

- **The interface is the same on both architectures** ([0008](0008-architecture-neutral-design.md)). Only the kernel's method differs.
- **The kernel parses no bytecode.** User mode reads the sleep values ([0032](0032-minimal-hardware-discovery.md)).
- **Holding the capability is a trust decision.** Its holder can stop the machine at any time.
- **A service can fail the machine on purpose,** as the root task does when an essential service fails ([0216](0216-essential-service-failure.md)).
- **The kernel gains one more control capability,** like the counter capability ([0113](0113-counter-capability.md)).
