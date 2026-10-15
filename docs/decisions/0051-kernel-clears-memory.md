# 0051. The kernel clears memory before it's made into anything new

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Memory passes from one program to another over time, and what one holder left in it must never reach the next. Memory is also reused within one holder, and that never involves the memory service:

- A program can make memory into a thread object for another program, take it back, and remake it as pages it can read.
- A client can take back memory it lent a service ([0050](0050-clients-pay.md)), and remake it the same way.

Only the kernel sees every reuse. seL4's kernel clears memory each time it's made into a new object.

## Options

1. **The kernel** clears memory whenever it's reused, before it becomes a new object.
2. **The memory service** clears memory before handing it on.
3. **The kernel by default,** with the memory service allowed to skip that for memory it has already cleared ahead of time.

## Decision

Option 1.

- The kernel clears memory before it's made into a new object, whether that's pages for a program or a kernel object.
- The memory service clears nothing.

Option 2 was chosen first. A consistency review on 2026-10-15, before anything was committed, found that reuse within one holder never reaches the memory service, so option 2 would leak data between programs. Option 1 replaced it.

## Consequences

- **One rule covers every path.** That includes reuse within one holder, memory the memory service takes back and hands on, and memory used for the first time after boot.
- **Keeping programs' data apart depends on the kernel alone,** not on any user-mode service.
- **Clearing costs time when memory is reused.** Large pieces are cleared in interruptible steps ([0060](0060-kernel-stack-per-core.md)).
- **The kernel briefly maps the pages it clears.** Otherwise it maps no program data ([0055](0055-kernel-maps-only-objects.md)).
- **Sharing or handing over memory doesn't clear it** ([0058](0058-shared-and-handed-over.md)), because nothing new is made from it.
- **Kernel objects start from cleared memory,** and the kernel also sets up every part of each object itself.
