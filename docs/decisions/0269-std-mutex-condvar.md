# 0269. std's `Mutex` and `Condvar` are built on parking

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

std's locks are built on parking, and parking on notifications, with no futex ([0177](0177-locks-on-parking.md)). 0177 says std's mutexes, condition variables and once-cells use std's queue-based versions. The library sources of the pinned release, 1.99.1, and of the nightly of 2026-10-16 have queue-based versions of `RwLock` and `Once` only. `Mutex` and `Condvar` have none: only versions built on a futex, on pthreads, or for one other system each ([0256](0256-audit-findings-as-evidence.md)). std's patch stays within what upstream would accept ([0187](0187-pinned-stable-toolchain.md)).

## Options

1. **Both built on parking.**
2. **`Mutex` as the queue-based `RwLock`'s write side,** with only `Condvar` new.

## Decision

Option 1.

- std's patch adds a `Mutex` and a `Condvar` built on thread parking, in the style of the queue-based `RwLock`.
- They're generic, for any target that has parking, and they're offered upstream with the targets ([0187](0187-pinned-stable-toolchain.md), [0251](0251-later-work.md)).
- They're tier 1 code in std's patch ([0242](0242-review-tiers.md)), built at step 14 ([0229](0229-std-stages.md)).
- This amends 0177, whose mutexes and condition variables are new code in std's patch, not upstream's.

## Consequences

- **0072 still holds.** Neither lock asks the kernel to read a lock word.
- **A lock pays only for what it does.** Unlike option 2, a `Mutex` carries no bookkeeping for readers.
- **Step 14 adds tier 1 code,** which its estimate now includes ([08-roadmap](../design/08-roadmap.md)).
- **Upstream could gain locks for targets without a futex,** if it takes them.
- **Step 14 is affected.**
