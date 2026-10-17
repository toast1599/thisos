# 0174. Random numbers come from a random service, and every program starts with a seed

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

std needs random numbers at startup, for its HashMap keys. Crates such as getrandom need a port ([0004](0004-not-unix.md)). No record said where random numbers come from. [0061](0061-no-randomization-yet.md) is about randomizing layouts, not this.

## Options

1. **A random service,** plus a seed in the startup data.
2. **The CPU's random instructions,** run by programs directly.
3. **A random service only.**

## Decision

Option 1.

- A random service is reached through a namespace entry, `svc/random` ([0131](0131-namespace-path-prefixes.md)). It's seeded at boot from the CPU's random instructions and from the firmware's random protocol.
- The loader writes a fresh seed into each program's startup data ([0146](0146-startup-data-block.md)), so std has its HashMap keys without making a call.
- This amends 0108, whose list gains the seed.

## Consequences

- **Every program has randomness from its first instruction.**
- **getrandom's port calls the random service.**
- **The firmware's random protocol is read by the boot loader,** while the firmware's boot services still run, and passed on in the boot information. Our fork of the crate gains that too ([0150](0150-bootloader-aarch64.md)). The root task passes the seed to the random service ([0159](0159-root-task-boot-information.md)).
- **Programs started before the random service runs** get seeds that the root task or the loader takes from the CPU's random instructions ([0160](0160-root-task-starts-four.md)).
- **A sandbox without the entry still has its seed,** but can't ask for more.
