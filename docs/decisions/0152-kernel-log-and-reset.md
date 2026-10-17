# 0152. The kernel writes its log to memory and a serial port, and resets the machine itself

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

When the kernel panics, it writes a message to its log output, then halts or reboots by a boot setting ([0110](0110-kernel-panic.md)). A fault in the root task is handled the same way ([0115](0115-root-task-fault-boot-setting.md)). Writing to a serial port and resetting the machine are platform code, but [0032](0032-minimal-hardware-discovery.md) and [0046](0046-scope-of-0008.md) limit the kernel's platform code to the memory map, the interrupt controller and the timer. The crate's boot information names neither ([0151](0151-boot-info-from-crate.md)).

## Options

1. **A serial port and the reset in the kernel,** amending 0032 and 0046.
2. **The log on the framebuffer** that the boot information hands over.
3. **The log in memory only,** with the reset method registered later by the root task.

## Decision

Option 1.

- The kernel writes its log to a ring buffer in memory, and to a debug serial port. That's COM1 on x86_64, the PC convention, which QEMU follows. On AArch64, it's the UART the firmware tables name.
- It reboots through the reset register in ACPI's FADT on x86_64, or with a triple fault where there's none. On AArch64, it uses PSCI.
- This amends 0032, whose list gains a serial port and the reset method, and 0046, whose limit on the kernel's platform code gains a write-only serial output and the reset.

## Consequences

- **A panic is visible from the kernel's first instruction,** on the serial port, before any user-mode program runs.
- **The serial output is write-only.** The kernel never reads from it, so it takes no input from outside.
- **The ring buffer is memory set aside at boot** ([0068](0068-no-kernel-heap.md)), which programs can only read. The root task receives a read-only capability to it ([0159](0159-root-task-boot-information.md)), and passes it to a log service.
- **The serial port is the kernel's.** Its I/O ports or device memory are left out of what the root task receives ([0159](0159-root-task-boot-information.md), [0168](0168-device-memory-and-ports.md)).
- **The kernel's platform code grows a little:** a write-only driver for each kind of UART it supports, and the reset. On x86_64, the kernel also reads ACPI's FADT, beside the tables it reads under [0151](0151-boot-info-from-crate.md).
- **0110's and 0115's log output is defined.**
