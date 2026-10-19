# 0245. Host tests cover every line of the generic kernel, and system calls are fuzzed from step 4

- **Status:** Accepted, amended by [0259](0259-outside-input-fuzzed.md)
- **Date:** 2026-10-19

## Context

The kernel has high test coverage, and fuzzing of the system-call interface, from the start ([0018](0018-kernel-assurance.md)). 0018 left the targets and the setup open. The generic kernel runs on a simulated machine ([0233](0233-simulated-machine.md)). Checked on 2026-10-19 with 1.99.1: line coverage, with `-Cinstrument-coverage`, is stable, and branch coverage needs `-Zcoverage-options=branch`, which `RUSTC_BOOTSTRAP` unlocks ([0187](0187-pinned-stable-toolchain.md)).

## Options

1. **Every line, with fuzzing from step 4.**
2. **90% of lines,** with the same fuzzing.
3. **No number.** Coverage is reported, and reviews judge it.

## Decision

Option 1.

- Host tests cover every line of the generic kernel, measured on the simulated machine. A line left out carries a stated reason in the code.
- Branch coverage is reported, but not required.
- QEMU tests reach every function of the architecture and platform code.
- Coverage-guided fuzzing of system calls runs on the simulated machine from step 4. It runs for a few minutes in each CI run, and for hours nightly. Its corpora are kept in the repository.
- From step 9, a program in QEMU fuzzes system calls on both architectures, with the kernel's invariant checks on.
- Each trusted-base service's protocol is fuzzed through its generated bindings ([0195](0195-bindings-safe-output.md)), from the step that builds the service.

## Consequences

- **No line of the generic kernel goes untested without a reason on record.**
- **The fuzzer explores interleavings of simulated cores** ([0233](0233-simulated-machine.md)).
- **Each trusted-base service's message handling is fuzzed** where clients could attack it ([0016](0016-threat-model-scope.md)).
- **0018's question on coverage targets and fuzzing is answered.**
