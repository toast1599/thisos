# 0105. A failed program's supervisor has its memory taken back, and decides on a restart

- **Status:** Accepted, amended by [0267](0267-supervisor-keeps-connect.md)
- **Date:** 2026-10-16

## Context

Taking back a program's memory destroys everything made from it, including its threads and address space ([0053](0053-revoke-destroys-derived.md)), which stops the program. [0082](0082-corrections.md) left open who has that done when a program fails. [04-execution](../design/04-execution.md) also asked whether failed drivers and services are restarted, as MINIX 3 does.

## Options

1. **Its supervisor** ([0104](0104-supervisor.md)) asks the memory service to take the memory back.
2. **The memory service** takes it back itself when a crash is reported to it.

## Decision

Option 1.

- A program's supervisor holds a capability that lets it ask the memory service to take back all of the program's memory, which stops the program.
- The supervisor decides whether to restart the program.

## Consequences

- **The kernel stays out of it.** Reclaiming is a revoke ([0053](0053-revoke-destroys-derived.md)) that user mode asks for. This settles the line in [0014](0014-microkernel.md) that 0082 corrected.
- **Restart policy belongs to each supervisor.** A supervisor can restart failed drivers and services, as MINIX 3 does.
- **The memory service still stops programs on its own** to free memory ([0049](0049-memory-fair-share.md)), and the supervisor is told ([0104](0104-supervisor.md)).
- **04's fault-isolation questions are answered.**
