# 0144. The system library uses the C calling convention, and only data, capabilities and regions cross between programs

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

[0007](0007-abi-calls-and-interfaces.md) was proposed on Day 1 for the kernel-mode design, and it was never accepted. Under the microkernel ([0014](0014-microkernel.md)) and registers-only calls ([0072](0072-registers-only.md)), two of its points no longer apply: kernel calls as ordinary function calls, and buffers passed between programs as a pointer and a length. Its other points were still open. Day 5 settles all of them: the system library ([0117](0117-system-library.md)), the IDL ([0142](0142-own-idl.md)) and versioning ([0143](0143-add-only-versioning.md)).

## Options

1. **Supersede it,** as [0014](0014-microkernel.md) superseded [0013](0013-isolation-domains.md), which was never accepted either, and carry over what still holds.
2. **Accept it, amended** by a corrections record like [0082](0082-corrections.md).

## Decision

Option 1. This supersedes 0007.

- **Calls into the system library** ([0117](0117-system-library.md)) use the C calling convention of each architecture, through `extern "C"`: System V AMD64 on x86_64, and AAPCS64 on AArch64. Three rules from 0007 apply on top:
  - strings are UTF-8, passed as a pointer and a length, with no NUL terminator;
  - errors are returned as values, with no errno;
  - there are no variadic functions.
- **Between programs,** only these cross an interface:
  - fixed-layout data, carried in messages ([0119](0119-message-eight-words.md));
  - capabilities ([0123](0123-capabilities-in-messages.md));
  - regions, passed as capabilities and shared, lent or handed over ([0050](0050-clients-pay.md), [0058](0058-shared-and-handed-over.md)).
- Pointers never cross between programs. Neither do Rust types such as `String`, `Vec` and `Box`, trait objects, generics, or enums that carry data.
- One IDL defines every interface ([0142](0142-own-idl.md)), and interfaces only grow ([0143](0143-add-only-versioning.md)).

## Consequences

- **All of 0007's points are settled:** its calling convention here and in [0117](0117-system-library.md), what crosses a boundary here, its IDL in [0142](0142-own-idl.md), and its versioning in [0143](0143-add-only-versioning.md).
- **Pointers stay inside a program.** The system library runs in the program, so its functions can take pointers to the program's own memory, while the kernel still sees only registers ([0072](0072-registers-only.md)).
- **Nothing is freed across an interface** ([0059](0059-caller-allocates.md)), as 0007 also required.
- **What the freeze covers for interfaces** is set by [0148](0148-freeze-covers-rules.md). It replaces 0007's freeze of version 1.
- **Records that cite 0007 as Proposed** find its points in these records.
