# 0124. Kernel calls whose data doesn't fit in registers are split, and the system library joins them

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The kernel carries data only in registers ([0072](0072-registers-only.md)), and a message carries up to 8 words ([0119](0119-message-eight-words.md)). Some kernel calls have more data than that. An example is reading all of a thread's registers, including its vector state, for an exception handler or a debugger ([0102](0102-exception-handler.md)). With wide vector registers, that's several kilobytes.

## Options

1. **Split by the library.** The kernel offers such calls as several calls that each fit in registers, and the system library joins the parts.
2. **A kernel-written buffer.** The kernel writes the result into a buffer the caller supplies, in one call. That ends 0072's rule.

## Decision

Option 1.

- The kernel offers such operations as several calls, each within registers, such as reading or writing a thread's registers in groups.
- The system library's function joins the parts into one structure in the caller's memory, so the stable ABI ([0117](0117-system-library.md)) shows a single function.
- A thread's registers can be read or written only while it's stopped, in a fault or suspended, so the parts agree.

## Consequences

- **0072 holds.** The kernel still never reads or writes program data.
- **Large reads cost several system calls.** Debuggers and exception handlers can afford it. The fault message itself carries what most handlers need ([0102](0102-exception-handler.md), [0133](0133-recovery-point-in-register.md)).
- **The split can change with any system image,** since only the library sees it.
- **05's question on calls that don't fit in registers is answered.**
