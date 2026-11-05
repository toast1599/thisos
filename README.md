# ThisOS

ThisOS is a general-purpose operating system written in Rust, built on a microkernel. The kernel manages memory, threads, handles, IPC and interrupt delivery. Drivers, filesystems and other services run as isolated user-space programs that reach only what they were handed. It runs on 64-bit hardware, starting with x86_64, and programs will target Rust's standard library. It has no libc and is not Unix-like.

## Status

The design for the first milestones is settled. Milestone 1 (booting the kernel) is done: the kernel boots under UEFI in QEMU and on the test laptop, logs to serial and a framebuffer console, prints a backtrace when it panics, and runs its tests in QEMU, in CI and on the laptop. Milestone 2 (the kernel core) is in progress.

- [docs/design.md](docs/design.md) describes the system and its milestones.
- [docs/adr/](docs/adr/) holds the decisions that are hard to reverse.
- [CONTRIBUTING.md](CONTRIBUTING.md) says how code is written, tested and decided.

## License

ThisOS is licensed under either of the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in ThisOS by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
