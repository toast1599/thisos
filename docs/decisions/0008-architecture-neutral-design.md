# 0008. Architecture-neutral design

- **Status:** Accepted, amended by [0082](0082-corrections.md)
- **Date:** 2026-10-14

## Context

Over its lifetime the OS is expected to run on more than one CPU architecture. Architecture-specific concepts tend to leak into generic code: privilege levels, page sizes, memory ordering, interrupt and timer hardware, how hardware is discovered. Once they leak, every new port means rewriting that code.

## Options

1. Design for one architecture and port to others later.
2. Keep the design and all generic code architecture-neutral, and confine architecture-specific concepts to architecture-specific code.

## Decision

Option 2. Nothing except architecture-specific code relies on any architecture-specific concept.

## Consequences

- The design documents describe concepts in neutral terms.
  - The design says **kernel mode** for the privilege level the kernel runs at: ring 0 on x86_64, EL1 on AArch64, S-mode on RISC-V ([0002](0002-kernel-mode-verified-execution.md)).
  - On every architecture, firmware or a hypervisor can run above kernel mode: SMM and VMX root on x86_64, EL2 and EL3 on AArch64, M-mode on RISC-V.
- Generic code and interface definitions can't assume any of the following:
  - a particular page size (AArch64 supports 4, 16 and 64 KiB pages)
  - a particular cache-line size
  - strong memory ordering. x86_64 orders memory accesses more strictly than AArch64 and RISC-V, so concurrent code that is correct only on x86_64 fails on the others.
  - that unaligned memory access works
  - a particular interrupt controller, timer, or hardware-discovery method (ACPI or device tree)
- The ABI's calling convention is already defined per architecture ([0007](0007-abi-calls-and-interfaces.md)). Interface definitions have to state integer widths, alignment and byte order explicitly.
