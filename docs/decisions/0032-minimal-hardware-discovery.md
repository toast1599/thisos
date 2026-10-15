# 0032. The kernel reads only the hardware information it needs to start

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

ACPI tables and device trees describe the hardware, and parsing them takes a lot of code that is easy to get wrong. To start, the kernel needs only three things: the memory map, the interrupt controller and a timer.

## Options

1. **The minimum in the kernel**, everything else parsed in user mode.
2. **The kernel parses everything** at boot.

## Decision

Option 1.

- The kernel reads only the memory map, and information about the interrupt controller and a timer.
- A user-mode service parses ACPI tables or device trees for everything else.

## Consequences

- **Even the minimum is architecture-specific.** It differs by architecture and platform, so that code is architecture-specific ([0008](0008-architecture-neutral-design.md)).
- **Drivers learn about devices from the user-mode service,** not from the kernel ([06-system](../design/06-system.md)).
- **The kernel's platform code stays small.** It is limited to the interrupt controller, the timer and reading the memory map. That narrows 01's question on how far 0008 reaches ([01-objective](../design/01-objective.md)).
