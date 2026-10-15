# 0002. All code runs in kernel mode, isolated by verification

- **Status:** Superseded by [0014](0014-microkernel.md)
- **Date:** 2026-10-14

## Context

Traditional operating systems isolate programs with hardware privilege levels: the kernel runs in kernel mode and programs run in user mode. The alternative is to run every program in kernel mode and get isolation by verifying each program before it is allowed to run.

Kernel mode is the privilege level the kernel itself runs at: ring 0 on x86_64, EL1 on AArch64, S-mode on RISC-V ([0008](0008-architecture-neutral-design.md)).

Prior art:
- SPIN (1995): type-safe extensions
- Singularity and Midori (Microsoft): software-isolated processes in kernel mode
- Verve: a verified type-safe OS
- Theseus and RedLeaf (2020): Rust, a single privilege level
- Tock: a hybrid of language and hardware isolation
- eBPF: verified code in the Linux kernel, used in production but restricted

Mechanisms raised so far for verification include a signed compiler, compile-time lints, and a signed build system. None has been chosen yet ([02-verification](../design/02-verification.md)).

## Options

1. Hardware isolation, the traditional model.
2. Every program in kernel mode, isolated by verification.
3. A hybrid of the two.

## Decision

Option 2. Every program runs in kernel mode, and no program runs until it has been verified. The goal is a security model under which it is mathematically impossible for a program to exploit the system or gain arbitrary memory access.

Memory safety and system safety are the priority. The speed gained by avoiding privilege transitions is a welcome side effect, not a goal. This is explicitly not a TempleOS-style design.

This is the end goal for the core system, which userspace will be built on top of.

## Consequences

- Verification is the only barrier. A flaw in the verifier, the compiler, or the trusted code compromises the whole machine. The trusted base must stay as small as possible and be audited heavily ([0005](0005-unsafe-code-policy.md)).
- The verification mechanism, and where it runs, are still open ([02-verification](../design/02-verification.md)).
- Some threats lie outside what the language can enforce, and the design still has to deal with them:
  - speculative execution (Spectre)
  - hardware faults (Rowhammer)
  - stack overflow
  - denial of service
  - cleanup after a program fails
- Kernel mode isn't the highest privilege level on any supported architecture: firmware or a hypervisor can run above it. That code is outside verification, so the threat model has to state how far it is trusted ([01-objective](../design/01-objective.md)).
