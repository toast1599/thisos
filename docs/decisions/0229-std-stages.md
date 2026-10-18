# 0229. std comes in four stages, each done when upstream's tests for it pass

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Programs target std, which is ported in stages ([0003](0003-rust-std-no-libc.md)), and all of std is supported in the end ([0182](0182-all-of-std.md)). 0182 left the order of the stages to the roadmap. The root task is a std program ([0159](0159-root-task-boot-information.md)). Third-party programs unwind on panic, and the OS's own services abort ([0109](0109-panic-strategy.md)). Checked on 2026-10-19, the library sources of the pinned release, 1.99.1, carry std's own tests: the tests in `library/std/tests`, 56 test modules inside std, and the `coretests` and `alloctests` crates.

## Options

1. **Four stages, checked by upstream's tests.**
2. **The same stages, checked by tests of our own.**
3. **As option 1, with unwinding only at the SDK step.**

## Decision

Option 1.

- **Stage 1, at step 10:** startup from the startup data ([0146](0146-startup-data-block.md)), allocation, arguments, environment variables, exit, and abort on panic.
- **Stage 2, at step 14:** threads, locks and parking ([0177](0177-locks-on-parking.md)), sleep and timeouts, thread-locals, time ([0173](0173-time-service-entry.md)), randomness ([0174](0174-random-service-and-seed.md)), standard streams ([0180](0180-streams-and-exit-status.md)), and unwinding ([0109](0109-panic-strategy.md)).
- **Stage 3, at step 19:** files, paths and the current directory ([0176](0176-paths-and-current-directory.md)), and processes, exit statuses and pipes ([0179](0179-command-copies-namespace.md), [0180](0180-streams-and-exit-status.md)).
- **Stage 4, at step 22:** networking ([0181](0181-network-service.md)).
- A stage is done when upstream's tests for its parts pass on both architectures, except tests of what thisOS reports as Unsupported ([0182](0182-all-of-std.md)).

## Consequences

- **std is checked against upstream's own expectations.**
- **Test programs unwind, like third-party programs.** Rust's test harness needs unwinding. Test images aren't shipped, so 0109's rule for the OS's own services holds ([0044](0044-system-image-is-os-shipped.md)).
- **Services built before stage 2 use the native API** for what stage 1 lacks, such as threads ([0145](0145-std-over-native-api.md)).
- **0182's question on the order of the stages is answered.**
