# 0259. Network packets, USB input and disk images are fuzzed

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

System calls are fuzzed, and so is each trusted-base service's protocol, through its generated bindings ([0245](0245-coverage-and-fuzzing.md)). Input from outside never comes through bindings: packets come from the wire, descriptors and transfers from a device, and a filesystem's image from a disk ([0257](0257-outside-attackers.md)). Rust's safety rules out memory corruption, but not hangs, panics, which stop the OS's own services ([0109](0109-panic-strategy.md)), or wrong state. Step 17 tests the filesystem's crash consistency, not its handling of hostile images ([08-roadmap](../design/08-roadmap.md)). The audit found this gap from 0245's text ([0256](0256-audit-findings-as-evidence.md)).

## Options

1. **Packets, USB input and disk images.**
2. **Packets and disk images,** with USB left to its review ([0258](0258-outside-input-tier-2.md)).
3. **Review only.**

## Decision

Option 1.

- From step 17, crafted disk images are fuzzed into the filesystem service.
- From step 18, USB descriptors and transfers are fuzzed into the USB core and its drivers, through a simulated host controller.
- From step 22, network packets are fuzzed into the network service.
- Each fuzzer runs for a few minutes in every CI run, and for hours nightly. Its corpora are kept in the repository, as for system calls ([0245](0245-coverage-and-fuzzing.md)).
- This amends 0245, whose fuzzing gains these.

## Consequences

- **Parsers of outside input are tested with what an attacker would send.**
- **The parsers are written so a fuzzer can drive them,** for example in generic crates that also build for the host ([0232](0232-traits-and-specific-crates.md)).
- **Steps 17, 18 and 22 are affected** ([08-roadmap](../design/08-roadmap.md)).
