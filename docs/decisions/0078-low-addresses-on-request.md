# 0078. Programs that need low addresses ask for them

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

The kernel turns on the widest addressing the CPU offers ([0063](0063-widest-addressing.md)). Some programs keep tags in the upper bits of pointers, as some JavaScript engines do, so they need addresses that stay low. Linux places memory below 47 bits unless a program asks for more.

## Options

1. **They ask.**
2. **Low by default,** with the full width only on request.

## Decision

Option 1.

- By default, memory can be placed anywhere in the lower half.
- A program that needs low addresses says so when it reserves address space, giving the highest address it can use.

## Consequences

- **No default limit on addresses,** in line with [0012](0012-no-memory-cap.md) and [0063](0063-widest-addressing.md).
- **Programs written for this OS ask explicitly** ([0015](0015-clean-apis.md)). Ports that assume 47-bit addresses add the request, or a compatibility layer adds it for them ([0006](0006-non-rust-software.md)).
- **The request is part of reserving address space,** which is designed with memory requests ([05-interfaces](../design/05-interfaces.md)).
- **03's low-address question is answered.**
