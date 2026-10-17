# 0182. All of std is supported, in stages, and allocation failure aborts

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Programs target std, which is ported in stages ([0003](0003-rust-std-no-libc.md)). A memory request that can't be met fails at the request, with an error the program can handle ([0048](0048-reserve-on-grant.md)). 0048 left what std then does to the std mapping.

## Options

1. **All of std, in stages,** with allocation failure aborting.
2. **The same, with allocation failure panicking,** so that programs that unwind can recover. Rust allows that only through an unstable flag on each program's build.

## Decision

Option 1.

- All of std is supported, built in stages ([0003](0003-rust-std-no-libc.md)).
- Calls with no equivalent here return `Unsupported` errors. That covers Unix-style permissions, symbolic links, and user and group names.
- Allocation failure aborts the program with a message, as on other targets. `try_reserve` and the other fallible methods return errors.

## Consequences

- **Programs behave as on other targets** when memory runs out.
- **Programs that must survive a failed allocation use the fallible methods.**
- **std's allocator reserves and backs its heap through the pager** ([0138](0138-pager-keeps-reservations.md)). A request that fails there reaches std as a failed allocation.
- **The order of the stages is the roadmap's** ([08-roadmap](../design/08-roadmap.md)).
- **0048's question on allocation failure is answered,** and so is 06's on which std features are supported.
