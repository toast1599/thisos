# Decision log

Each decision gets one record, numbered in order. A record covers the context, the options that were considered, the decision itself, and its consequences.

## Status values

- **Accepted**: approved by the maintainer. Binding.
- **Proposed**: written up but not yet approved. Not binding.
- **Superseded**: replaced by a later record, which it links to.

Until the freeze on 2026-10-20, an Accepted decision can still be changed. Changing one means writing a new record that supersedes it; records are never edited after the fact.

Open questions are listed in the [design documents](../design/), grouped by topic.

## Records

| # | Decision | Status | Date |
|---|---|---|---|
| [0001](0001-general-purpose-os.md) | General-purpose production OS | Accepted | 2026-10-14 |
| [0002](0002-kernel-mode-verified-execution.md) | All code runs in kernel mode, isolated by verification | Superseded by [0014](0014-microkernel.md) | 2026-10-14 |
| [0003](0003-rust-std-no-libc.md) | Rust is the system language; the Rust standard library replaces libc | Accepted | 2026-10-14 |
| [0004](0004-not-unix.md) | Not Unix-like: `target_family` is not `unix` | Accepted | 2026-10-14 |
| [0005](0005-unsafe-code-policy.md) | Unsafe code exists only in one closed, audited trusted set | Superseded by [0017](0017-unsafe-code-kernel-and-services.md) | 2026-10-14 |
| [0006](0006-non-rust-software.md) | Non-Rust software is supported later; route still open | Accepted | 2026-10-14 |
| [0007](0007-abi-calls-and-interfaces.md) | ABI: C calling convention, interfaces defined in an IDL, add-only versioning | Proposed | 2026-10-14 |
| [0008](0008-architecture-neutral-design.md) | Architecture-neutral design: only architecture-specific code relies on architecture-specific concepts | Accepted | 2026-10-14 |
| [0009](0009-x86-64-first.md) | x86_64 is the first architecture | Accepted | 2026-10-14 |
| [0010](0010-64-bit-only.md) | 64-bit architectures only | Accepted | 2026-10-14 |
| [0011](0011-aarch64-second.md) | AArch64 is the second architecture | Accepted | 2026-10-15 |
| [0012](0012-no-memory-cap.md) | No fixed limit on program memory | Accepted | 2026-10-15 |
| [0013](0013-isolation-domains.md) | Isolation domains: one shared system domain, one checked address space per program | Superseded by [0014](0014-microkernel.md) | 2026-10-15 |
| [0014](0014-microkernel.md) | Microkernel: a minimal, audited kernel in kernel mode; everything else in user mode | Accepted | 2026-10-15 |
| [0015](0015-clean-apis.md) | APIs and ABIs are designed for this system: inspired by others, never bound to them | Accepted | 2026-10-15 |
| [0016](0016-threat-model-scope.md) | Threat model: speculative leaks between programs and denial of service are in scope | Accepted | 2026-10-15 |
| [0017](0017-unsafe-code-kernel-and-services.md) | Unsafe code: one closed, audited set for the kernel and the OS's own services | Accepted | 2026-10-15 |
| [0018](0018-kernel-assurance.md) | Kernel assurance: review, tests and fuzzing from the start; formal proofs later | Accepted | 2026-10-15 |
| [0019](0019-kernel-minimality.md) | Kernel minimality: anything that can reasonably run in user mode does | Accepted | 2026-10-15 |
| [0020](0020-capabilities.md) | Authority comes from capabilities | Accepted | 2026-10-15 |
| [0021](0021-ipc-simple-upgradable.md) | IPC starts simple and modular, and is designed to be upgraded | Accepted | 2026-10-15 |
| [0022](0022-explicit-spawn.md) | Programs start other programs by explicit spawn | Accepted | 2026-10-15 |
| [0023](0023-no-quotas.md) | No quotas: a program's resources are limited only by the hardware and what's available | Accepted | 2026-10-15 |
