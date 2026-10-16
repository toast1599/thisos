# 0125. The kernel never returns with `sysret` to an address outside the program half

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

On Intel CPUs, `sysret` to a non-canonical address raises a fault in kernel mode while the program's stack pointer is still loaded (CVE-2012-0217). Exception handlers can resume a thread with changed registers ([0102](0102-exception-handler.md)), so user mode controls the address a thread returns to. The canonical range depends on the address width chosen at boot ([0063](0063-widest-addressing.md)). AArch64's `eret` has no such hazard.

## Options

1. **Check, then fall back.** The kernel checks the return address before every `sysret`, and uses `iretq` when the check fails, as Linux does.
2. **Always `iretq`.** The kernel never uses `sysret`.

## Decision

Option 1, with two guards.

- The kernel refuses to set any thread's instruction pointer outside the program half ([0066](0066-kernel-upper-half.md)). A handler that tries gets an error.
- Before every `sysret`, the kernel also checks that the return address lies in the program half, for the address width chosen at boot ([0063](0063-widest-addressing.md)). If it doesn't, the kernel returns with `iretq`.

## Consequences

- **The hazard can't be reached** even if one of the guards has a bug.
- **Returns keep `sysret`'s speed** in the normal case. The check is one comparison.
- **This is x86_64 code,** and lives in the kernel's x86_64 part ([0008](0008-architecture-neutral-design.md), [0046](0046-scope-of-0008.md)).
- **05's question on the `sysret` hazard is answered.**
