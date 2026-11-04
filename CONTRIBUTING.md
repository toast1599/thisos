# Contributing to ThisOS

This file covers how ThisOS code is written, tested and decided. What the system is lives in [docs/design.md](docs/design.md).

## Decisions

Every choice that has a reasonable alternative goes to the maintainer before its code is written. That covers implementation details as well as design. The answers then go to one of three places:

- Answers that change what the system is, or an interface between crates, update `docs/design.md`.
- Answers that are hard to reverse also get a record in `docs/adr/`. These are the syscall ABI, the kernel's object model, IPC semantics and policies like the ones in this file.
- Everything else is stated in the commit message.

An ADR has a status line, a date, and three sections: Context, Decision and Consequences. It is under a page long, and is numbered in sequence (`0005-short-name.md`). A provisional ADR says what will confirm it.

## Work items

Each milestone is a GitHub milestone, and each of its work items is a GitHub issue. An item's issue is opened when work on it starts, and the commit that finishes it closes it. A work item is small enough to review in one sitting, and ends with something that builds or boots.

## Toolchain

The whole repository uses one pinned, dated nightly, set in `rust-toolchain.toml` and bumped by hand. It is the newest nightly that has rustfmt, clippy, Miri, `rust-src` and the `x86_64-unknown-none` and `x86_64-unknown-uefi` targets. The kernel builds for the prebuilt `x86_64-unknown-none` target.

`rust-toolchain.toml` uses the minimal profile, and lists those components and targets explicitly.

Every crate uses the 2024 edition. Move to the 2027 edition once it is released, expected some time in 2027.

Build profiles:

- `panic = "abort"` in every profile, because the kernel can't unwind.
- Frame pointers are forced for target builds in `.cargo/config.toml`, so a panic can print a backtrace.
- Debug info is on in dev and release.
- Dev builds use `opt-level = 1`. At level 0 the kernel is slow and its stack frames are large.

There are two workspaces:

- The root workspace holds the target crates. Its `.cargo/config.toml` defaults to `x86_64-unknown-none`.
- `xtask/` is its own workspace for host tooling, run as `cargo xtask`.

## xtask

- `cargo xtask build` builds the kernel and writes a UEFI GPT disk image to `target/thisos.img` with `bootloader`'s `DiskImageBuilder`, then prints the image's path and its build ID. It uses the dev profile; `--release` builds release.
  The build ID is `ThisOS <version> <commit> <profile> <time>`, for example `ThisOS 0.0.0 f172dcc-dirty debug 2026-11-04T09:35Z`: the commit is `git describe --always --dirty`, and the time is when xtask built it, in UTC. xtask passes everything after the version to the kernel build as `THISOS_BUILD`. The kernel logs the whole ID as its first line, or `unknown build` in place of the rest when built without xtask. The image holds the ID in `thisos-build.txt` at the root of its FAT partition. `--tests` builds the laptop test image instead (see below), and its ID has `tests` after the profile, as in `ThisOS 0.0.0 3ddbe39 debug tests 2026-11-04T15:00Z`.
- `cargo xtask run` builds, then boots the image in QEMU with a window for the framebuffer and serial output on the terminal. QEMU's monitor is on Ctrl-Alt-2 in that window. `--gdb` starts QEMU's gdb stub on TCP port 1234 and starts the CPU paused.

QEMU runs:

- the q35 machine, one CPU and 512 MiB of RAM
- KVM with the host's CPU when `/dev/kvm` is usable, and TCG emulation otherwise
- the image as USB storage on an xHCI controller, the way the laptop boots its stick
- `-no-reboot`, so a triple fault closes QEMU and doesn't loop
- `cargo xtask flash` builds, then writes the image, or with `--tests` the test image, to the USB stick for a laptop run (see below).
- `cargo xtask test` runs the host tests of every `lib/*` crate, then the QEMU tests with `cargo test -p thisos-kernel`. It is the only way to run the QEMU tests: it sets cargo's runner for `x86_64-unknown-none` through `CARGO_TARGET_X86_64_UNKNOWN_NONE_RUNNER` to its own hidden `qemu-test` subcommand, because cargo starts the runner in `kernel/`, where the `xtask` alias's relative manifest path doesn't resolve. A bare `cargo test -p thisos-kernel` runs the kernel ELF on the host, which dies with SIGSEGV.
- `cargo xtask ci` runs locally everything CI runs, in CI's order, and stops at the first failure: rustfmt on both workspaces; clippy with `-D warnings` on the root workspace for `x86_64-unknown-none`, on the kernel's tests for the same target, on each `lib/*` crate for the host with its tests, and on xtask with its tests; xtask's host tests; `cargo xtask test`; Miri on each `lib/*` crate; `cargo deny` on both workspaces; `cargo xtask build`. It needs `cargo-deny` installed (`cargo install cargo-deny --locked`) and checks for it first.
- `cargo xtask font` rasterizes `xtask/assets/JetBrainsMono-Regular.ttf` (v2.304, under the SIL Open Font License in `xtask/assets/OFL.txt`) with `fontdue` into one 8-bit coverage bitmap per size in `lib/console/src/font/`, and writes the `font.rs` that embeds them. The TTF, the bitmaps and `font.rs` are committed, so ordinary builds never run the rasterizer. It prints any glyphs it had to clip to their cell.

OVMF comes from the `ovmf-prebuilt` crate at a pinned release, `edk2-stable202605-r1`, which xtask caches in `target/ovmf/`. Each run copies the firmware's variables file to `target/ovmf-vars.fd`, so every boot starts from the same state.

## Unsafe code

`thisos-arch-x86_64` is the only crate allowed unsafe code. Every other crate starts with `#![forbid(unsafe_code)]`. This is an attribute and not a workspace lint, because Cargo can't combine a crate's own `[lints]` table with the workspace's. Adding a crate to the exception list is a decision like any other.

Neither `forbid(unsafe_code)` nor clippy's unsafe lints check code that a macro from another crate expands into, and a `macro_rules!` body is only checked where it is expanded. So unsafe code reaches the kernel through `thisos_arch_x86_64::entry!` and the `bootloader_api::entry_point!` it calls, and no lint checks the SAFETY comment inside `entry!`. A dependency whose macros a forbidding crate uses is reviewed as unsafe code. A macro in the arch crate that expands to unsafe code follows the rules below as if its expansion were written in the arch crate.

Inside the exception list:

- Every `unsafe` block has a `// SAFETY:` comment saying which invariant holds and why.
- Every `unsafe fn` has a `# Safety` section in its docs.
- Each unsafe block does one unsafe operation.
- Each unsafe operation is wrapped once, in a type whose API is safe and whose invariants are encoded in the type. Examples: physical and virtual address newtypes, a frame that isn't `Copy`, a guard that restores interrupts when dropped, typed MMIO registers.

Tests back up a `// SAFETY:` argument; they don't replace it.

## Lints

Target crates (kernel, hal, arch, abi, `lib/*`, `user/*`) share `[workspace.lints]` in the root `Cargo.toml`.

These are denied:

- `unsafe_op_in_unsafe_fn`
- `clippy::undocumented_unsafe_blocks`
- `clippy::missing_safety_doc`
- `clippy::multiple_unsafe_ops_per_block`
- `clippy::cast_possible_truncation`, `clippy::cast_sign_loss` and `clippy::as_conversions`: use `From` and `TryFrom`
- `missing_docs` on public items
- `clippy::unwrap_used` and `clippy::expect_used`, in the kernel and arch crates only, through a `#![deny(...)]` attribute in each crate root

`clippy::pedantic` is on as a warning group. Four of its lints are allowed: `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc` and `missing_panics_doc`. CI passes `-D warnings`, so pedantic warnings fail the build. Test code is linted too: the kernel's `cfg(test)` code, each `lib/*` crate's tests and xtask's tests. `clippy.toml` adds ThisOS to the identifiers that doc comments may write without backticks.
xtask uses clippy's defaults. Formatting is rustfmt's defaults. Identifiers, comments, docs and messages use American spelling, as in "color" and "initialize".

## Tests

- **Host tests.** Generic logic lives in `lib/*` crates and runs under `cargo test` on the host. Every `lib/*` crate also runs its tests under Miri.
- **QEMU tests.** In-kernel tests are `#[test_case]` functions using `custom_test_frameworks`. `cargo test` builds one kernel ELF per test binary. xtask is the cargo runner: it writes each ELF into an image next to it, `<elf>.img`, and boots it headless in QEMU with serial on standard output. The kernel logs `test <name>` before each test and `<n> tests passed` after the last, to serial and the screen. It ends on a summary: `summary:`, then a line per test saying `ok`, `FAILED` or `not run`, then PASS or FAIL, four times the font's size on the screen. A panic outside any test puts `panic before the tests started` in place of the test lines, or `panic after the last test` after them. Then it exits QEMU through `isa-debug-exit` at port 0xf4: 0x10 for success, which QEMU turns into exit status 33, and 0x11 for failure, status 35. Only 33 is a pass, because `-no-reboot` turns a triple fault into status 0. The kernel writes the port only when CPUID reports a hypervisor, which QEMU does with KVM and with TCG; on the laptop, where nobody knows what 0xf4 decodes to, it halts after the summary. Until milestone 2 adds the first faulting test, the tests live in the kernel binary under `cfg(test)`, and `thisos-kernel` has no library.
- **Failures.** A failing test's panic report is followed by `test <name> failed` and the summary, and the kernel exits QEMU with failure. The tests after it in that binary don't run; other binaries still run.
- **Faulting tests.** A test that has to fault, such as a page fault or a stack overflow, goes in a `tests/` file of its own.
- **Shared support.** Test files share their entry point and panic handler through one support module, so that the combined laptop image can include them.
- **Hangs.** At milestone 1, xtask kills QEMU after 60 s per test binary and fails it. From milestone 2, a watchdog on the LAPIC timer stops a hung test and names it.

## Real hardware

Anything that passes in QEMU must also pass on the test laptop described in `docs/design.md`. Laptop runs happen at every milestone and after any change to arch or driver code. A milestone isn't done until it passes there.

`cargo xtask flash` writes the image to a USB stick, erasing it. It takes `--release`, `--tests` and `--device PATH`. It runs as you, and runs `dd` and `sgdisk` through `sudo`. It:

- only considers whole disks on USB that are removable or hotplug, and never the internal disk. With one such disk it uses it; with several, `--device` picks one.
- refuses a disk with no `/dev/disk/by-id` link, and names the disk by that link in every command after choosing it. A stick swapped while flash waits for an answer takes its link with it, so `dd` fails and no other disk is written.
- shows the stick's path, by-id link, model, serial and size, and the build ID the stick holds. It reads `thisos-build.txt` from the stick's FAT partition, mounting it with `udisksctl` if it isn't mounted. Then it always asks before writing: external USB disks report hotplug too, so the question is what tells the stick from a backup drive left plugged in.
- refuses a stick smaller than the image, and unmounts the stick's partitions first.
- writes with `dd`, drops the stick's cached pages with `blockdev --flushbufs` so that the read comes from the stick, then reads the image back and compares it byte for byte.
- moves the backup GPT to the end of the stick with `sgdisk -e`, checks the table with `sgdisk -v`, and powers the stick off.

`sgdisk -e` is needed because the image is only a few MiB, so after `dd` its backup GPT sits near the start of the stick. The laptop's firmware doesn't list a stick in that state, and Linux warns about it.

Boot the stick from the laptop's F12 menu, with Secure Boot off.

A panic's backtrace lists link addresses. To name them, run `addr2line` on the kernel ELF that went into the image, with the profile it was built in:

```
addr2line -f -C -e target/x86_64-unknown-none/debug/thisos-kernel 0x28af3 0x2bc5b
```

`llvm-addr2line` takes the same arguments.

`cargo xtask build --tests` writes the kernel's test binary into `target/thisos-tests.img`, and `cargo xtask flash --tests` writes that to the stick. It finds the binary in the JSON messages of `cargo test --no-run`. The image ends on the summary screen described under Tests, with the panic report above it when a test fails, then halts. Once milestone 2 adds faulting tests, it becomes one combined image holding every test that doesn't fault, plus one image per faulting test file.

## CI

GitHub Actions runs on every push and pull request:

- rustfmt
- clippy with `-D warnings`
- host tests
- Miri
- the QEMU tests, under TCG
- `cargo deny`
- an image build

`.github/workflows/ci.yml` has one job on `ubuntu-24.04` that runs `cargo xtask ci`, so CI and a local run do the same checks. It installs the toolchain from `rust-toolchain.toml` with the runner's rustup, `cargo-deny` 0.20.2 as a prebuilt binary through `taiki-e/install-action`, and QEMU from Ubuntu's `qemu-system-x86` package, which the runner image lacks. Every action is pinned to a full commit SHA, with its tag in a comment. The workflow only reads the repository, and a new push cancels a run still going on the same ref.

CI caches the cargo registry and both target directories, keyed on `rust-toolchain.toml` and both `Cargo.lock` files. The cache holds `target/ovmf/` too: fetching OVMF from GitHub can take minutes, and the cache means CI pays that only when the pinned release changes.

## Dependencies

Dependencies come from crates.io. Each new dependency is reviewed when it is added, and adding one is a decision. Version requirements are caret ranges such as `"0.11"`. `Cargo.lock` is committed, and pins the exact versions. A `cargo update` is a change reviewed like any other. `cargo deny` checks advisories, licenses and sources against `deny.toml`, which both workspaces share. It allows exactly the licenses the dependencies use today, so a dependency with a new license fails until the license is added there, as part of that dependency's review. Unmaintained crates fail like other advisories; an ignored advisory is listed in `deny.toml` with its reason. Crates come from crates.io only, and duplicate versions are a warning. Nothing is vendored.
