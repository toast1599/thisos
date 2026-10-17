# 0168. Device memory and I/O ports are capabilities of their own

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Drivers need device memory mapped for them ([0014](0014-microkernel.md)). Ordinary memory is made into kernel objects and program pages, and the kernel clears it first ([0051](0051-kernel-clears-memory.md)). On x86_64, some devices still use I/O ports, such as the CMOS clock and legacy serial ports.

## Options

1. **Device-memory capabilities, and I/O ports through system calls.**
2. **Device-memory capabilities, and I/O ports through the CPU's per-program permission bitmap.**

## Decision

Option 1. seL4 handles both this way.

- Device memory is a kind of memory capability of its own, covering physical ranges outside RAM.
- It can be mapped uncached or write-combining, as its holder asks. It's never made into kernel objects or program pages, and the kernel never clears it.
- On x86_64, I/O ports are capabilities for ranges of ports, used through system calls.

## Consequences

- **Drivers get only their own device's ranges,** from the device manager ([0172](0172-device-manager.md)).
- **Device memory is never mapped executable.** Its rights are read and write ([0128](0128-rights-and-badges.md)).
- **Each port access costs a system call,** which the few devices that still use ports can afford. No bitmap is kept or switched for each program.
- **The kernel's own devices are left out:** its interrupt controller, its timer and its serial port ([0152](0152-kernel-log-and-reset.md), [0159](0159-root-task-boot-information.md)).
- **Port capabilities are architecture-specific** ([0008](0008-architecture-neutral-design.md)). AArch64 has no I/O ports.
