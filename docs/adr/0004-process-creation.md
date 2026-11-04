# 0004. The kernel loads only init; a user-space loader builds every other process

- **Status:** Accepted
- **Date:** 2026-11-04

## Context

A kernel `spawn(elf, handles)` syscall is the quickest way to get a second process running. It would also make ELF parsing and loading policy permanent parts of the kernel's ABI. The other choice is to have the kernel load only the first process, and give user space syscalls general enough to build the rest.

## Decision

The kernel parses one ELF file: `init`'s, from the bootloader's ramdisk. Every other process is built by a loader in user space, using these general syscalls:

- create an address space
- map memory objects into it
- create a thread in it
- start that thread with the handles it should hold

## Consequences

- The kernel keeps a minimal ELF parser for `init` only. All program loading policy stays out of the kernel's ABI.
- `init` holds the first loader. The syscalls for building another process's address space are needed by milestone 4.
- Starting a process takes several syscalls in place of one. If that proves too slow, it gets a new record.
