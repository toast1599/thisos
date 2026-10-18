# 0233. The kernel's generic part also runs on the host, on a simulated machine

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

The kernel has strict review, high test coverage and fuzzing from the start ([0018](0018-kernel-assurance.md)). Its generic code reaches the hardware only through traits that architecture and platform crates implement ([0232](0232-traits-and-specific-crates.md)). Coverage measurement, coverage-guided fuzzing and Miri run on the host. The kernel has one lock, which long operations yield at their interruption points ([0095](0095-one-kernel-lock.md), [0218](0218-lock-yield.md)).

## Options

1. **A simulated machine.** The generic kernel also builds for the host, against a simulated machine.
2. **QEMU only.** Kernel tests run only in a test build of the kernel in QEMU.
3. **Host tests for pure parts,** the modules that touch no hardware.

## Decision

Option 1.

- The generic kernel also builds for the host, against a simulated machine. The simulator implements the architecture and platform traits in ordinary host code: memory is a buffer, page tables and cores are data, and test code makes the system calls.
- The simulator steps several simulated cores, in an order that the test or the fuzzer chooses.
- Unit tests, coverage measurement, fuzzing and Miri run the generic kernel there ([0235](0235-kinds-of-tests.md), [0245](0245-coverage-and-fuzzing.md)).
- The simulator is a generic crate with no unsafe code, and it never ships.
- QEMU tests cover the architecture and platform code.

## Consequences

- **The generic kernel's tests are fast and repeatable,** and run on every change.
- **Interleavings across cores are tested on purpose,** including the lock's yields ([0218](0218-lock-yield.md)).
- **The work between interruption points can be counted exactly** ([0246](0246-interruption-stretch-limit.md)).
- **The traits stay narrow enough to simulate,** which keeps hardware detail out of the generic kernel ([0008](0008-architecture-neutral-design.md)).
