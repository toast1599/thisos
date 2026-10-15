# 0040. Programs are signed, and the device owner's policy decides which signers may run

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Under [0014](0014-microkernel.md), isolation no longer depends on checking programs. A signature can still show who built a program.

## Options

1. **A signature, plus the owner's policy.**
2. **A signature from a trusted signer, always required.**
3. **No check at all.**

## Decision

Option 1.

- Programs carry a signature showing who built them.
- The device owner's policy decides which signers' programs may run, as with code signing on macOS.

## Consequences

- **Signatures are for provenance and policy,** not for isolation.
- **Several details are open** ([02-verification](../design/02-verification.md)):
  - the default policy;
  - how keys are managed;
  - who signs the system image;
  - where the check runs, in the loader or in a separate service.
- **Packages are what get signed** ([0041](0041-immutable-packages.md)).
