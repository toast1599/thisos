# 0133. Accessors keep their recovery point in a register

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Touching memory that its payer took back is an error, not a crash ([0065](0065-lost-memory-is-an-error.md)). The pager declines the fault, which then goes to the thread's exception handler, and the handler resumes the thread at a recovery point ([0106](0106-pager-passes-on-faults.md)). The handler is often another program, such as the supervisor, and it can't read the faulting program's memory. Linux keeps its recovery points in a table inside the kernel image.

## Options

1. **In a register.** Accessors hold their recovery address in a fixed register while they touch the memory.
2. **A table in the program,** read by an exception handler inside the program.
3. **Registered with the kernel** before each access.

## Decision

Option 1.

- The accessors for lent and shared memory ([0065](0065-lost-memory-is-an-error.md)) hold the address to resume at in a fixed register while they touch that memory.
- The fault message carries that register. Any handler, inside the program or outside it, can then resume the thread there with an error, without reading the program's memory.
- Which register is documented for each architecture with the system library ([0117](0117-system-library.md)).
- std implements the handler's side once.

## Consequences

- **Supervisors built on std recover their programs' accessors for free.**
- **The kernel gains nothing** ([0019](0019-kernel-minimality.md)). The register is a convention between accessors and handlers.
- **Accessors are written in assembly for each architecture,** so the compiler can't move an access outside the stretch where the register holds. In the OS's own services, they belong to the audited set ([0017](0017-unsafe-code-kernel-and-services.md)).
- **A handler that finds no recovery address** treats the fault as a crash.
- **The fault message gains a word,** within the 8 a message carries ([0119](0119-message-eight-words.md)).
- **05's question on delivering memory loss as an error is answered.**
