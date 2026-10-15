# 0072. System calls and IPC carry data only in registers; the kernel never reads program data

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Under [0055](0055-kernel-maps-only-objects.md), the kernel maps no program data. When a message sits in program memory, the kernel has to either reach that memory or avoid it. seL4 maps a buffer page for each thread. Messages already have a small fixed maximum size ([0036](0036-ipc-message-max.md)), and bulk data goes through shared regions ([0058](0058-shared-and-handed-over.md)).

## Options

1. **Registers only.**
2. **Copy through a kernel buffer,** reaching each side through its own mappings.
3. **IPC buffers as kernel objects,** as in seL4.
4. **Leave it to the first IPC design.**

## Decision

Option 1.

- System calls and IPC carry data only in registers.
- Anything larger goes through shared regions ([0058](0058-shared-and-handed-over.md)).
- The kernel never reads program data. It writes program memory only to clear it ([0051](0051-kernel-clears-memory.md)), and at boot to set up the root task ([0052](0052-user-mode-page-tables.md)).

## Consequences

- **The kernel has no code that follows program pointers.** Nothing in it checks pointers from programs or recovers from faults on them, which keeps it smaller and easier to check ([0018](0018-kernel-assurance.md), [0019](0019-kernel-minimality.md)).
- **Registers cap the message size.** 0036's maximum is what fits in registers on every architecture: around 8 to 10 words (64 to 80 bytes) on x86_64. The exact size comes with the first IPC design ([05-interfaces](../design/05-interfaces.md)).
- **Some calls have to be designed around this.** Calls whose data doesn't fit, such as reading all of another thread's registers, need another shape ([05-interfaces](../design/05-interfaces.md)).
- **Faults reach pagers in registers.** A fault's address and kind fit in a few registers ([0027](0027-user-mode-pagers.md)).
- **Ending this rule takes a new record.** A later IPC version that wanted larger messages through the kernel would end this rule, so it would need a new record ([0021](0021-ipc-simple-upgradable.md)).
