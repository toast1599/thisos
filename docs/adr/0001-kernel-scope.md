# 0001. The kernel owns memory, scheduling, handles, IPC and interrupt delivery

- **Status:** Accepted
- **Date:** 2026-11-04

## Context

A microkernel can be very small, as seL4 is. In seL4 user space builds page tables, handles page faults, manages physical memory and sets scheduling policy, and the kernel has no heap. That works for a verified kernel built by a team. For a system built by one person it means nothing useful runs until several user-space services exist: a root task, a memory manager, a pager, a loader and a scheduler. Zircon and Redox keep these mechanisms in the kernel and still run drivers, filesystems and networking in user space. That is where most of a microkernel's fault isolation comes from.

## Decision

The kernel owns:

- physical memory, page tables and a kernel heap
- threads and a preemptive scheduler
- handles and the kernel objects they name
- IPC channels and shared memory
- delivery of interrupts to user space

Drivers, filesystems, networking and all other services run in user space. A mechanism moves out of the kernel later only with evidence that it should.

## Consequences

- The first user program can run at milestone 3, with no user-space resource managers.
- The kernel is larger than seL4's. Its unsafe code stays in the arch crate, and the generic kernel forbids unsafe code.
- Page faults are handled in the kernel. Pagers in user space, if ever wanted, need a new record.
