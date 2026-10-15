# 05. System calls, IPC and the stable ABI

- **Day:** 5 (Sun 2026-10-18)
- **Status:** Not started

## Covers

The binary interface between programs and the kernel and between programs, how interfaces are defined and versioned, and the security model at the interface level.

## Decided

- [0015](../decisions/0015-clean-apis.md) APIs and ABIs are designed for this system: inspired by others, never bound to them
- [0020](../decisions/0020-capabilities.md) Authority comes from capabilities
- [0021](../decisions/0021-ipc-simple-upgradable.md) IPC starts simple and modular, and is designed to be upgraded
- [0028](../decisions/0028-namespace-at-spawn.md) Programs find services through a namespace handed over at spawn
- [0029](../decisions/0029-users-as-capabilities.md) A user is the set of capabilities a login service grants
- [0030](../decisions/0030-filesystem-decides-access.md) The filesystem service decides who may open stored data
- [0036](../decisions/0036-ipc-message-max.md) One IPC message has a small fixed maximum size; larger data goes through shared memory
- [0050](../decisions/0050-clients-pay.md) Clients pay for the state services keep for them
- [0058](../decisions/0058-shared-and-handed-over.md) Programs share bulk data through regions that are shared or handed over
- [0059](../decisions/0059-caller-allocates.md) Memory that crosses an interface is allocated by the caller, and its use moves only by handover
- [0064](../decisions/0064-memory-belongs-to-payer.md) Memory belongs to the program it was granted to; lending, sharing and handover move only access
- [0065](../decisions/0065-lost-memory-is-an-error.md) Touching memory that was taken back is an error, not a crash
- [0072](../decisions/0072-registers-only.md) System calls and IPC carry data only in registers; the kernel never reads program data

One proposal is waiting for approval:

- [0007](../decisions/0007-abi-calls-and-interfaces.md) ABI: C calling convention, interfaces defined in an IDL, add-only versioning (**Proposed**)

0007 was written for kernel mode. Under [0014](../decisions/0014-microkernel.md), its first point (kernel calls as ordinary function calls) no longer applies. Its other points still do.

## Open questions

- **IDL:** our own, modeled on FIDL, or WIT ([0007](../decisions/0007-abi-calls-and-interfaces.md)). It has to express memory a client lends a service [0050](../decisions/0050-clients-pay.md), handover [0058](../decisions/0058-shared-and-handed-over.md), and caller-provided memory for results of unknown size, including how the caller learns how much to provide [0059](../decisions/0059-caller-allocates.md). Its bindings include the accessors that turn memory taken back into errors [0065](../decisions/0065-lost-memory-is-an-error.md).
- **Capability design** ([0020](../decisions/0020-capabilities.md)). What rights does a capability carry, and how is it passed and revoked? Rights include execute on memory [0054](../decisions/0054-kernel-enforces-execute.md), and revoking follows the kernel's records of what was made from what [0053](../decisions/0053-revoke-destroys-derived.md). Also whether a thread's pager can be changed after spawn, and by whom [0056](../decisions/0056-pager-chosen-at-spawn.md). Also the namespace's format, and whether a program can pass parts of it on [0028](../decisions/0028-namespace-at-spawn.md).
- **The role of std.** Either std is the native API, or it's a thin, documented layer over the native API. The choice is narrowed to these two. With capabilities ([0020](../decisions/0020-capabilities.md)), it leans towards a layer over the native API.
- **System calls.** Under [0014](../decisions/0014-microkernel.md), programs enter the kernel with the architecture's system-call instruction: `syscall` on x86_64, `svc` on AArch64. Which registers carry the call number, the arguments and the results on each architecture ([0008](../decisions/0008-architecture-neutral-design.md))? All data travels in registers [0072](../decisions/0072-registers-only.md). On x86_64, returning to a program with `sysret` at an address outside the canonical range is a known hazard on Intel CPUs. How the kernel avoids it, as Linux does, is decided here.
- **The first IPC design** ([0021](../decisions/0021-ipc-simple-upgradable.md)). Which simple starting point: synchronous calls between two programs (the L4 and seL4 model), asynchronous channels (Zircon), or shared memory? And how do programs reach IPC so the mechanism underneath can change? IPC cost largely decides performance in a microkernel. Estimated from Day 2's measurements, a round trip costs at least about 530 cycles on the development machine ([02-verification](02-verification.md)). Messages have a small fixed maximum [0036](../decisions/0036-ipc-message-max.md), and its exact size is set here. Data travels only in registers [0072](../decisions/0072-registers-only.md), so the maximum is what fits in registers on every architecture, around 8 to 10 words on x86_64.
- **Identity without user IDs.** Under [0029](../decisions/0029-users-as-capabilities.md), services see capabilities, not users. How do audit logs, file ownership and similar needs record who acted?
- **Asking for predictor flushes.** How a program asks to have the branch predictors flushed when it's switched away from [0035](../decisions/0035-predictor-flush-on-request.md).
- **Calls that don't fit in registers** ([0072](../decisions/0072-registers-only.md)). How calls whose data is larger than the registers, such as reading all of another thread's registers, are shaped.
- **Delivering memory loss as an error** ([0065](../decisions/0065-lost-memory-is-an-error.md)). How a fault on memory that its payer took back reaches the code that touched it.
- **Memory requests** ([0064](../decisions/0064-memory-belongs-to-payer.md), [0071](../decisions/0071-three-memory-programs.md)). How a program asks the memory service for memory, how its payer account is identified, and how the loader and pagers draw on a program's memory to back pages and make page tables [0052](../decisions/0052-user-mode-page-tables.md). Reserving address space includes asking for low addresses [0078](../decisions/0078-low-addresses-on-request.md).
- **Memory notices** ([0076](../decisions/0076-memory-notice-deadline.md)). How the memory service's notice to give memory back reaches a program.
