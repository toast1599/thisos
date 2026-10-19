# 0016. Threat model: speculative leaks between programs and denial of service are in scope

- **Status:** Accepted, amended by [0202](0202-offline-attackers.md) and [0257](0257-outside-attackers.md)
- **Date:** 2026-10-15

## Context

[01-objective](../design/01-objective.md) left the scope of the threat model open for four threats:
- speculative-execution leaks between programs (Spectre);
- denial of service;
- hardware faults such as Rowhammer;
- firmware and hypervisors running above kernel mode.

[0001](0001-general-purpose-os.md) already requires the threat model to assume programs that don't trust each other.

## Options

Each threat is either in scope, meaning the OS defends against it, or out of scope, meaning it's documented as a limitation.

## Decision

- **In scope:**
  - speculative-execution leaks between programs, and from programs into the kernel;
  - denial of service between programs.
- **Out of scope:**
  - Rowhammer and similar hardware faults;
  - firmware and hypervisors running above kernel mode, which are trusted.

## Consequences

- **Spectre defence is best-effort.** No CPU guarantees freedom from speculative leaks. The defence rests on separate address spaces and the hardware's user/kernel defences ([0014](0014-microkernel.md)). It may also need branch-predictor flushes when switching between programs; one costs 545 cycles ([02-verification](../design/02-verification.md)).
- **No program can starve the others.** It must not be able to take all the CPU time, memory or kernel objects. That shapes scheduling and resource accounting ([04-execution](../design/04-execution.md)). Quotas are ruled out ([0023](0023-no-quotas.md)), so the defence has to come from elsewhere.
- **The hardware keeps memory intact.** Against Rowhammer, the OS relies on the hardware, for example ECC memory, and adds no software mitigation.
- **Firmware and hypervisors are trusted.** They belong to the trusted base, along with the boot path and the hardware ([0014](0014-microkernel.md)).
- **Leaks within a program are the program's concern.** For example, a browser has to keep its own sites apart.
