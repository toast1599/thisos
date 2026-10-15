# 0044. Everything in the system image counts as shipped by the OS

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0017](0017-unsafe-code-kernel-and-services.md) holds the OS's own drivers and services to its unsafe-code rule, but left "shipped by the OS" undefined.

## Options

1. **Everything in the system image.**
2. **A named list** of core services.

## Decision

Option 1.

- Whatever the project builds and ships in the system image ([0042](0042-updates.md)) is held to 0017.
- Anything installed later is a third-party program.

## Consequences

- **The line is mechanical.** It is whatever goes into the image.
- **0017 covers every program in the image,** not only core services.
- **Separately published programs are third-party.** A program the project publishes outside the image follows normal Rust practice, like any third-party program.
