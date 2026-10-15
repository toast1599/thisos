# 0024. At boot, one root task receives every initial capability

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

With capabilities ([0020](0020-capabilities.md)) and no ambient authority, something has to hold all authority when the system starts.

## Options

1. **Root task.** The kernel hands every initial capability to one first user-mode program, the root task, which passes narrower ones on. This is seL4's model.
2. **Boot policy in the kernel.** The kernel creates the first services itself and assigns their capabilities from a built-in boot policy.

## Decision

Option 1.

- The kernel starts exactly one user-mode program, the root task, and hands it every capability that exists at boot.
- The root task starts the rest of the system and gives each program only what it needs.

## Consequences

- **Boot policy lives in user mode.** All the kernel does at boot is create the root task ([0019](0019-kernel-minimality.md)).
- **The root task joins the trusted base.** It holds all authority until it hands it on, so it belongs to the trusted base along with the kernel and the boot path ([0014](0014-microkernel.md)).
- **It follows 0017.** As one of the OS's own services, it is bound by [0017](0017-unsafe-code-kernel-and-services.md).
- **The start sequence is a boot question.** Which programs the root task starts, and in what order, belongs with the boot path ([06-system](../design/06-system.md)).
