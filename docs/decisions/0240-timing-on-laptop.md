# 0240. Benchmarks and time budgets run on the development laptop

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

GitHub's runners are virtual machines, so KVM there is nested, and timings are noisy ([0237](0237-public-repository-and-ci.md)). Benchmarks are tracked over time ([0235](0235-kinds-of-tests.md)), and the kernel has a time budget for its longest stretch between interruption points ([0246](0246-interruption-stretch-limit.md)). The development laptop runs QEMU under KVM ([0223](0223-qemu-and-laptop-checkpoints.md)).

## Options

1. **On the laptop,** at checkpoints and for kernel changes that affect them.
2. **On GitHub, nightly,** with the noise accepted.
3. **On a self-hosted runner** on the laptop. GitHub warns against those for public repositories, since pull requests could run code on them.

## Decision

Option 1.

- Benchmarks and time budgets run on the development laptop under KVM, at each checkpoint, and for kernel changes that may affect them.
- GitHub's runners run correctness tests only.

## Consequences

- **Timing comes from one steady machine,** so results compare across runs.
- **No outside code runs on the laptop** through CI.
- **Timing isn't checked on every change,** so a tier 1 review asks for a timing run where a change may affect it ([0242](0242-review-tiers.md)).
