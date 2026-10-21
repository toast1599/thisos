# thisOS

thisOS is a general-purpose operating system built on a microkernel. The kernel itself is very small: it runs in kernel mode, and it's heavily audited and tested. Everything else runs in user mode on top of it, drivers, filesystems and system services included, and the hardware isolates programs from each other. We designed the APIs and ABIs for this system. Existing systems inspired them, but they're never bound to one for compatibility. Rust is the system language, and programs target the Rust standard library. There's no libc.

It's called thisOS because this OS differs from the traditional ones: its system language is Rust, where OSes such as Linux distributions, Windows and macOS use C.

## Status

**Week 1 (2026-10-14 → 2026-10-19) was for design only.** On 2026-10-19, at the end of Day 8, the design and the roadmap froze together ([0255](docs/decisions/0255-freeze.md)). From here on, the only way to change a frozen decision is a new record that cites what building found ([0201](docs/decisions/0201-changes-after-freeze.md)). The design was audited on the same day, after the freeze, and its checked findings count as such evidence ([0256](docs/decisions/0256-audit-findings-as-evidence.md)). They brought records 0256 to 0271.

On 2026-10-20, Day 9, building began with step 1, and what it found showed the roadmap needed a redesign. The roadmap reopened, while the rest of the design stayed frozen ([0272](docs/decisions/0272-roadmap-reopened.md)), and records 0273 to 0291 redesigned it. It froze again with the tag roadmap-2 ([0292](docs/decisions/0292-roadmap-refreezes.md)). Building resumes at step 1, and [docs/progress.md](docs/progress.md) records each working day: what was designed, added and done, with the evidence ([0279](docs/decisions/0279-progress-log.md)).

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

[08-roadmap](docs/design/08-roadmap.md) has the exact roadmap. It takes the project from an empty repository to a first release of the whole design, in 24 steps, through eight milestones ([0222](docs/decisions/0222-roadmap-to-first-release.md), [0273](docs/decisions/0273-whole-design-in-milestones.md), [0275](docs/decisions/0275-eight-milestones.md)). Every step opens with a design pass, and its code lands as small work items, each reviewed on its own ([0278](docs/decisions/0278-design-pass-work-items.md)). Every step gets built and tested in QEMU on x86_64, and from step 8 on, on AArch64 as well.

Each milestone ends in a working image that's reviewed, passes a written security pass, and is tagged. The development laptop boots it from a USB stick ([0274](docs/decisions/0274-milestone-definition.md), [0276](docs/decisions/0276-milestone-security-pass.md)). Milestone images are development images, for test machines only ([0284](docs/decisions/0284-development-images.md)).

| Milestone | Steps |
|---|---|
| m1. Foundations, on x86_64 | Two spikes, std as a patch and a user-mode IOMMU; 1–3: the repository, the build tool and its checks; booting the kernel, from a signed image; the kernel's base |
| m2. The kernel's objects, on x86_64 | 4–6: memory, capabilities and revoke; address spaces and threads; system calls, the IDL compiler and the system library |
| m3. User mode on both architectures | 7–9: the boot loader for AArch64; the first user-mode program; IPC, faults and interrupts |
| m4. The core services | 10–14: std's first stage and the root task; the scheduler; the memory service; the default pager, the loader and the boot manifest; the log, time and random services, with std's second stage |
| m5. Multicore and devices | 15–16: hardware discovery and multicore; the device manager, the PCI bus driver and the IOMMU service |
| m6. Storage, packages and updates | 17–20: NVMe, block storage and the filesystem; USB storage; packages, with std's third stage; updates, fallback and Secure Boot |
| m7. Users and the network | 21–22: the console, login and the shell; the network, with std's fourth stage |
| m8. The first release | 23–24: third-party programs; the first release |

We size each step by its code and the review that code needs, and promise no dates. Each milestone records the pace we actually measure, and the remaining work is forecast from it ([0277](docs/decisions/0277-sized-work-no-dates.md)). Third-party crates come from trusted publishers, and their vulnerabilities are tracked, so review goes to our own code ([0285](docs/decisions/0285-trusted-publishers.md)).

## License

thisOS is licensed under either of the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option ([0199](docs/decisions/0199-license.md)).

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in thisOS by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions ([0200](docs/decisions/0200-contributions.md)).

Contributions come in as pull requests, and a maintainer reviews them ([0238](docs/decisions/0238-pushes-and-pull-requests.md)).

## Security

If you find a security problem, please report it privately, through GitHub's private vulnerability reporting ([0239](docs/decisions/0239-security-reports.md)).
