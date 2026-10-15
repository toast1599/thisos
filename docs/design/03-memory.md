# 03. Memory model

- **Day:** 3 (Fri 2026-10-16)
- **Status:** Not started

## Covers

Address spaces, memory layout, allocation, and how programs share data.

## Decided

- [0012](../decisions/0012-no-memory-cap.md) No fixed limit on program memory
- [0014](../decisions/0014-microkernel.md) Microkernel: a minimal, audited kernel in kernel mode; everything else in user mode. Each program has its own address space.
- [0025](../decisions/0025-user-mode-memory.md) Physical memory is managed from user mode
- [0026](../decisions/0026-programs-pay-kernel-objects.md) Programs pay for their kernel objects with memory they hold
- [0027](../decisions/0027-user-mode-pagers.md) Page faults are handled by user-mode pagers

## Open questions

- **Kernel mapping.** Under [0014](../decisions/0014-microkernel.md), each program has its own address space. Does the kernel stay mapped in every address space, reachable only in kernel mode, or is it unmapped while user code runs? Unmapping is required on CPUs affected by Meltdown. It adds two page-table switches to every system call, at about 158 cycles each with PCID ([02-verification](02-verification.md)).
- **Stack overflow.** Safe Rust detects stack overflow at runtime, not at compile time. For user-mode programs, a guard page and a fault delivered to the program's pager [0027](../decisions/0027-user-mode-pagers.md) are enough. The kernel itself still requires:
  - guard pages
  - stack probes enabled in the target spec
  - fault handlers that run on their own stacks
- **Layout and allocation.** The physical and virtual memory layout. Under [0025](../decisions/0025-user-mode-memory.md), allocation policy lives in a user-mode memory service: how does it divide memory between programs?
- **Reclaiming memory.** When memory is taken back from a program, the kernel objects made from it [0026](../decisions/0026-programs-pay-kernel-objects.md) have to be destroyed safely. How?
- **Pagers.** Who is each program's pager [0027](../decisions/0027-user-mode-pagers.md), and how is a pager kept from faulting on its own memory?
- **Sharing data between programs.** Copying messages through IPC, shared memory, or moving pages from one address space to another. Messages have a small maximum size [0036](../decisions/0036-ipc-message-max.md), so bulk data goes through shared memory.
- **Memory that crosses interfaces.** Who allocates it and who frees it ([0007](../decisions/0007-abi-calls-and-interfaces.md)).
