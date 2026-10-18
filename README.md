# thisOS

thisOS is a general-purpose operating system built on a microkernel. The kernel itself is very small: it runs in kernel mode, and it's heavily audited and tested. Everything else runs in user mode on top of it, drivers, filesystems and system services included, and the hardware isolates programs from each other. We designed the APIs and ABIs for this system. Existing systems inspired them, but they're never bound to one for compatibility. Rust is the system language, and programs target the Rust standard library. There's no libc.

It's called thisOS because this OS differs from the traditional ones: its system language is Rust, where OSes such as Linux distributions, Windows and macOS use C.

## Status

**Week 1 (2026-10-14 → 2026-10-19) was for design only.** On 2026-10-19, at the end of Day 8, the design and the roadmap froze together ([0255](docs/decisions/0255-freeze.md)). From here on, the only way to change a frozen decision is a new record that cites what building found ([0201](docs/decisions/0201-changes-after-freeze.md)).

Every decision is written down in the [decision log](docs/decisions/README.md), and only the ones marked **Accepted** are binding.

## Week 1 plan

| Day | Date | Topic | Document |
|---|---|---|---|
| 1 | Wed 14 Oct | Objective, scope, threat model, trust model | [01-objective](docs/design/01-objective.md) |
| 2 | Thu 15 Oct | Trusted base, toolchain trust, unsafe policy | [02-verification](docs/design/02-verification.md) |
| 3 | Thu 15 Oct | Memory model | [03-memory](docs/design/03-memory.md) |
| 4 | Fri 16 Oct | Programs, isolation, scheduling, fault handling | [04-execution](docs/design/04-execution.md) |
| 5 | Sat 17 Oct | System calls, IPC and the stable ABI | [05-interfaces](docs/design/05-interfaces.md) |
| 6 | Sat 17 Oct | Boot, user-mode drivers, std mapping, non-Rust software | [06-system](docs/design/06-system.md) |
| 7 | Sun 18 Oct | Toolchain, build system, license, final review | [07-engineering](docs/design/07-engineering.md) |
| 8 | Mon 19 Oct | Implementation roadmap: the exact plan for building every part; engineering practices such as modularity and testing; freeze | [08-roadmap](docs/design/08-roadmap.md) |

## Roadmap

[08-roadmap](docs/design/08-roadmap.md) has the exact roadmap. It takes the project from an empty repository to a first release of the whole design, in 24 steps and 7 phases ([0222](docs/decisions/0222-roadmap-to-first-release.md), [0227](docs/decisions/0227-steps-and-phases.md)). Every step gets built and tested in QEMU on x86_64, and from step 8 on, on AArch64 as well. When a phase ends, the development laptop boots that phase's image ([0223](docs/decisions/0223-qemu-and-laptop-checkpoints.md)).

| Phase | Steps |
|---|---|
| 1. Foundations, on x86_64 | 1–3: the repository, the build tool and its checks; booting the kernel; the kernel's base |
| 2. The kernel's objects, on x86_64 | 4–6: memory, capabilities and revoke; address spaces and threads; system calls, the IDL compiler and the system library |
| 3. User mode on both architectures | 7–9: the boot loader for AArch64; the first user-mode program; IPC, faults and interrupts |
| 4. The core services | 10–14: std's first stage and the root task; the scheduler; the memory service; the default pager, the loader and the boot manifest; the log, time and random services, with std's second stage |
| 5. Multicore and devices | 15–16: hardware discovery and multicore; the device manager, the PCI bus driver and the IOMMU service |
| 6. Storage, packages and verified boot | 17–20: NVMe, block storage and the filesystem; USB storage; packages, with std's third stage; verified boot and updates |
| 7. Users, the network and the first release | 21–24: the console, login and the shell; the network, with std's fourth stage; third-party programs; the first release |

We estimate each step from the size of its code and the review that code needs. Checkpoints then calibrate those estimates against the pace we actually measure ([0249](docs/decisions/0249-estimates-and-checkpoints.md)).

## License

thisOS is licensed under either of the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option ([0199](docs/decisions/0199-license.md)).

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in thisOS by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions ([0200](docs/decisions/0200-contributions.md)).

Contributions come in as pull requests, and a maintainer reviews them ([0238](docs/decisions/0238-pushes-and-pull-requests.md)).

## Security

If you find a security problem, please report it privately, through GitHub's private vulnerability reporting ([0239](docs/decisions/0239-security-reports.md)).
