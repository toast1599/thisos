# 0232. Generic crates state their hardware needs as traits, and specific crates implement them

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Each crate of the OS's own code is marked as generic, or as specific to one architecture or platform, and the build rejects architecture-specific constructs in generic crates ([0197](0197-architecture-check.md)). 0197's check reads source and resolved paths. A generic crate could still depend on an architecture crate through a target-specific dependency in its manifest, which the check doesn't see. The kernel uses no trait objects or function pointers ([0198](0198-stack-depth-checker.md)). It reads its interrupt controller and timer from the firmware tables at boot ([0151](0151-boot-info-from-crate.md)).

## Options

1. **Traits, with specific crates on top.** Generic crates define traits, and architecture and platform crates implement them and build the binaries.
2. **Facade crates.** A generic crate depends on a facade, which picks the architecture crate in its manifest.
3. **Review only.**

## Decision

Option 1.

- Generic crates state what they need from the hardware as traits, used with static dispatch ([0198](0198-stack-depth-checker.md)).
- Architecture and platform crates implement those traits, and build the binaries. Dependencies point from specific crates to generic ones.
- The build rejects a generic crate that depends on an architecture- or platform-specific crate. This amends 0197, whose check also covers dependencies.
- A vendored crate counts as generic if its review finds that it builds and behaves the same on every supported architecture, through a portable path wherever it has architecture-specific code ([0243](0243-audit-records-and-crate-reviews.md)).
- Each architecture has one kernel binary. Its platform code is chosen at boot from the firmware tables ([0151](0151-boot-info-from-crate.md)), through enums.

## Consequences

- **No manifest can route around 0197.**
- **The same generic code runs on a simulated machine,** behind the same traits ([0233](0233-simulated-machine.md)).
- **One kernel per architecture boots every supported platform of it.**
- **Static dispatch keeps the stack checker's call graph complete** ([0198](0198-stack-depth-checker.md)).
