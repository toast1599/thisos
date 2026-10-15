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

One proposal is waiting for approval:

- [0007](../decisions/0007-abi-calls-and-interfaces.md) ABI: C calling convention, interfaces defined in an IDL, add-only versioning (**Proposed**)

0007 was written for kernel mode. Under [0014](../decisions/0014-microkernel.md), its first point (kernel calls as ordinary function calls) no longer applies. Its other points still do.

## Open questions

- **IDL:** our own, modeled on FIDL, or WIT ([0007](../decisions/0007-abi-calls-and-interfaces.md)).
- **Capability design** ([0020](../decisions/0020-capabilities.md)). What rights does a capability carry, and how is it passed and revoked? Also the namespace's format, and whether a program can pass parts of it on [0028](../decisions/0028-namespace-at-spawn.md).
- **The role of std.** Either std is the native API, or it's a thin, documented layer over the native API. The choice is narrowed to these two. With capabilities ([0020](../decisions/0020-capabilities.md)), it leans towards a layer over the native API.
- **System calls.** Under [0014](../decisions/0014-microkernel.md), programs enter the kernel with the architecture's system-call instruction: `syscall` on x86_64, `svc` on AArch64. Which registers carry the call number, the arguments and the results on each architecture ([0008](../decisions/0008-architecture-neutral-design.md))?
- **The first IPC design** ([0021](../decisions/0021-ipc-simple-upgradable.md)). Which simple starting point: synchronous calls between two programs (the L4 and seL4 model), asynchronous channels (Zircon), or shared memory? And how do programs reach IPC so the mechanism underneath can change? IPC cost largely decides performance in a microkernel. Estimated from Day 2's measurements, a round trip costs at least about 530 cycles on the development machine ([02-verification](02-verification.md)). Messages have a small fixed maximum [0036](../decisions/0036-ipc-message-max.md), and its exact size is set here.
- **Identity without user IDs.** Under [0029](../decisions/0029-users-as-capabilities.md), services see capabilities, not users. How do audit logs, file ownership and similar needs record who acted?
- **Asking for predictor flushes.** How a program asks to have the branch predictors flushed when it's switched away from [0035](../decisions/0035-predictor-flush-on-request.md).
