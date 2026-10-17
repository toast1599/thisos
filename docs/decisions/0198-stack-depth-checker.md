# 0198. A checker on the linked kernel bounds its stack depth and rejects recursion

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The kernel has one stack per core and uses no recursion, and the build checks its worst-case stack depth ([0060](0060-kernel-stack-per-core.md)). Fault handlers run on stacks of their own ([0060](0060-kernel-stack-per-core.md)). Which tool does the check was open. Checked on 2026-10-18:

- rustc's `-Zemit-stack-sizes` records each function's frame size in the object files, and the record survives linking with our toolchain ([0187](0187-pinned-stable-toolchain.md)).
- cargo-call-stack, the existing tool, was last tested with a nightly from 2022. It needs fat LTO, handles indirect calls imperfectly, and can't account for interrupt and fault handlers.

## Options

1. **Our own checker on the kernel binary.** Frame sizes come from `-Zemit-stack-sizes`, and the call graph from the linked kernel's machine code.
2. **cargo-call-stack.**
3. **Measured in tests.** Stacks are painted, and the deepest use is recorded during tests and fuzzing.

## Decision

Option 1.

- The build tool checks the linked kernel. It takes each function's frame size from `-Zemit-stack-sizes`, and builds the call graph from the kernel's machine code.
- Any cycle in the graph fails the build, so recursion is rejected.
- The kernel makes no indirect call the checker can't resolve. It uses no trait objects and no function pointers, and its log formatting avoids `core::fmt`'s dynamic dispatch. An indirect call the checker can't resolve fails the build.
- Each kernel stack is checked against every entry path that runs on it: each core's stack against system calls, interrupts and faults from user mode, and each fault stack against its handlers.

## Consequences

- **The bound is computed, not sampled.** It holds for paths that no test reaches.
- **The check covers `core`,** which the kernel builds from source with the same flags ([0189](0189-kernel-none-targets.md)).
- **Kernel code is written for the checker.** Static dispatch replaces trait objects, which also keeps the code simpler to prove later ([0018](0018-kernel-assurance.md)).
- **Each kernel stack is sized from the checker's result.**
- **0060's question on the tool is answered.**
