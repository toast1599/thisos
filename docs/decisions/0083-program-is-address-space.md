# 0083. To the kernel, a program is an address space and the threads bound to it

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

Under [0014](0014-microkernel.md), a program is a user-mode process: an address space and its threads. Several records depend on the kernel knowing what a program is:

- [0031](0031-user-mode-loader.md) requires a capability for creating programs. The root task holds it from boot ([0024](0024-root-task.md)), so it's a kernel capability. But [0026](0026-programs-pay-kernel-objects.md) lets any program make kernel objects from memory it holds, and nothing said what the capability gates.
- [0033](0033-cpu-fairness.md) shares CPU time between programs. If the kernel saw only threads, a program could enlarge its share by making more of them.
- [0034](0034-smt-trusted-pairs.md) pairs programs on a core, and [0080](0080-predictor-flush-both-ways.md) flushes on switches between programs.

## Options

1. **The address space is the program.**
2. **A program object of its own,** like Zircon's process, tying together an address space, threads and per-program settings.
3. **No kernel notion of a program,** as in seL4: only threads, address spaces and capability spaces.

## Decision

Option 1.

- To the kernel, a program is an address-space object and the threads bound to it.
- Where [0033](0033-cpu-fairness.md), [0034](0034-smt-trusted-pairs.md) and [0080](0080-predictor-flush-both-ways.md) refer to a program, they follow its address space.
- Making an address space needs 0031's capability for creating programs. Making threads in an address space a program already holds doesn't.

## Consequences

- **The kernel gains no object type for programs** ([0019](0019-kernel-minimality.md)).
- **0031's capability gates making address spaces.** The root task holds it at boot and decides who else gets it ([0031](0031-user-mode-loader.md)).
- **Switching between programs means switching between address spaces,** which settles which switches 0080's flushes apply to.
- **Running code in a separate address space needs the capability,** for example to sandbox part of a program, or a request to the loader.
- **What counts as one share is still open** ([04-execution](../design/04-execution.md)).
- **Whether a program's threads share one capability space** is part of the capability design ([05-interfaces](../design/05-interfaces.md)).
