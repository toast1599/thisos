# 0002. Typed handles with a small set of rights, moved when sent

- **Status:** Accepted
- **Date:** 2026-11-04

## Context

Programs need a way to name kernel objects, and the system needs a way to limit what each program can reach. The choices considered were these:

- Zircon-style handles with about a dozen rights per object type
- seL4 capability spaces with derivation trees and recursive revoke
- Redox-style resources opened by path

Any of these can work. The goal here is a model that is simple, safe and explicit.

## Decision

- **Typed.** Each handle names one kernel object of a fixed type, such as a channel, memory object, thread, process or interrupt.
- **Rights.** A handle carries rights from a fixed set: read, write, execute (memory objects only), duplicate and transfer. Rights can be removed and never added.
- **Moved on send.** Sending a handle over a channel moves it out of the sender. Sharing an object needs an explicit duplicate.
- **No ambient authority.** A new process holds only the handles its creator passes to it. The kernel has no global names.
- **Lifetime.** An object lives as long as some handle names it.
- **Rust API.** In user-space Rust, a handle is a non-`Copy` type such as `Handle<Channel>` that closes when dropped.

## Consequences

- There is no derivation tree, so revoking access that was already handed out needs a separate mechanism, such as a revocable proxy object. None is designed yet.
- Read-only sharing of memory uses a handle without the write right, not a separate object type.
- How a handle is represented, and the syscalls that create, duplicate and close handles, are still open. Both are due at milestones 3 and 4.
