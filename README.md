# Untitled OS

> Working title. The project name hasn't been decided yet. It will also be the OS name in the Rust target (`<arch>-unknown-<name>`).

A general-purpose microkernel operating system. A very small kernel runs in kernel mode, and it is heavily audited and tested. Everything else runs in user mode on top of it, including drivers, filesystems and system services, and the hardware isolates programs from each other. Its APIs and ABIs are designed for this system: inspired by existing systems, but never bound to one for compatibility. Rust is the system language, and programs target the Rust standard library. There is no libc.

## Status

**Week 1 (2026-10-14 → 2026-10-20) is for design only. No code is written.** Everything stays open to change during this week. At the end of the week the design is frozen.

Every decision is written down in the [decision log](docs/decisions/README.md). Only decisions marked **Accepted** are binding.

## Week 1 plan

| Day | Date | Topic | Document |
|---|---|---|---|
| 1 | Wed 14 Oct | Objective, scope, threat model, trust model | [01-objective](docs/design/01-objective.md) |
| 2 | Thu 15 Oct | Trusted base, toolchain trust, unsafe policy | [02-verification](docs/design/02-verification.md) |
| 3 | Fri 16 Oct | Memory model | [03-memory](docs/design/03-memory.md) |
| 4 | Sat 17 Oct | Programs, isolation, scheduling, fault handling | [04-execution](docs/design/04-execution.md) |
| 5 | Sun 18 Oct | System calls, IPC and the stable ABI | [05-interfaces](docs/design/05-interfaces.md) |
| 6 | Mon 19 Oct | Boot, user-mode drivers, std mapping, non-Rust software | [06-system](docs/design/06-system.md) |
| 7 | Tue 20 Oct | Engineering, final review, freeze | [07-engineering](docs/design/07-engineering.md) |

## Roadmap after the freeze

These are rough estimates. They will be revisited at the freeze.

| When | Kernel | Rust standard library |
|---|---|---|
| Weeks 2–3 | Toolchain, build, boot in QEMU, serial output, panic and logging, test harness | Kernel builds on `core` only |
| Weeks 3–5 | Physical and virtual memory: memory capabilities, kernel objects, page-table objects, revoke | — (the kernel has no heap and stays on `core`) |
| Weeks 5–7 | Interrupts, timers, scheduler, context switching | — |
| Weeks 7–9 | System calls and IPC, loader, first user-mode program on x86_64 and AArch64 | Custom target, std built with `-Zbuild-std`. Minimal std: allocation, stdout, args, exit, panics. |
| Weeks 9–12 | User-mode drivers, filesystem, init, shell | Threads, thread-local storage, locks, time |
| Month 3+ | SMP, networking | Filesystem, networking, process spawning. Ports of key crates. |
| Later | WebAssembly runtime for software not written in Rust | Get the target accepted upstream in rust-lang/rust |
