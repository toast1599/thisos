# 0082. Corrections to 0003, 0006, 0008, 0014 and 0077

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

A consistency review on 2026-10-16 found lines in Accepted records that later records overtook. Records are never edited after the fact, so a later record has to correct them.

## Options

1. **Amend, with a status note.** One record corrects the named lines, and each corrected record's status links to it.
2. **Amend, with status unchanged.** The same record, but only the index and the design docs link to it, as with [0046](0046-scope-of-0008.md).
3. **Supersede and restate.** Each affected decision gets a new record that restates it in full.
4. **Leave them as history.** The design docs note which lines later records overtook.

## Decision

Option 1. Each corrected record keeps its decision, and its status becomes "Accepted, amended by 0082". These lines are corrected:

- **[0003](0003-rust-std-no-libc.md), Consequences.**
  - "So the toolchain has to be pinned ([0005](0005-unsafe-code-policy.md))": 0005 is superseded. Pinning is set by [0037](0037-kernel-toolchain.md), which covers the kernel's toolchain.
  - "Rebuilding and re-verifying every program, and that update mechanism still has to be designed": programs are no longer verified ([0014](0014-microkernel.md)). Updates follow [0042](0042-updates.md): a std fix reaches the OS's own programs with the next system image, and a third-party program when its publisher rebuilds it.
- **[0006](0006-non-rust-software.md), Consequences.** "The code generator has to belong to the trusted set", and whether to reserve a place for it: under 0014, a JIT engine is an ordinary program, so its code generator doesn't have to be trusted. JIT engines make memory executable with the capability from [0043](0043-executable-memory-capability.md), and the kernel enforces it ([0054](0054-kernel-enforces-execute.md)).
- **[0008](0008-architecture-neutral-design.md), Consequences.** "The ABI's calling convention is already defined per architecture ([0007](0007-abi-calls-and-interfaces.md))": 0007 is only Proposed. System calls carry their data in registers ([0072](0072-registers-only.md)), and which registers carry what on each architecture is open ([05-interfaces](../design/05-interfaces.md)).
- **[0014](0014-microkernel.md), Consequences.** "The kernel reclaims its resources": the kernel reclaims nothing on its own. A failed program's memory comes back when it's revoked ([0025](0025-user-mode-memory.md), [0053](0053-revoke-destroys-derived.md)), and who has that done is open ([04-execution](../design/04-execution.md)).
- **[0077](0077-swap-in-default-pager.md), Decision and Consequences.** "Pages written out are encrypted" and "Storage never holds program data in the clear": both apply to pages the default pager writes out. Programs with their own pagers, which 0077 lets page out their own memory, aren't bound by them.

## Consequences

- **The five records stay Accepted,** and their status links here.
- **The decision log gains a status value:** "Accepted, amended by", for a record that a later one corrects in part.
- **Only 0077's correction is a decision of its own.** It settles that the encryption rule covers the default pager's swap. The other corrections restate what later records already decided.
