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

- `cargo xtask build` builds the kernel and writes a UEFI GPT disk image to `target/thisos.img` with `bootloader`'s `UefiBoot`, then prints the image's path. It uses the dev profile; `--release` builds release. `--tests` builds the laptop test images instead (see below).
- `cargo xtask run` builds, then boots the image in QEMU with a window for the framebuffer and serial output on the terminal. QEMU's monitor is on Ctrl-Alt-2 in that window. `--gdb` starts QEMU's gdb stub on TCP port 1234 and starts the CPU paused.

QEMU runs:

- the q35 machine, one CPU and 512 MiB of RAM
- KVM with the host's CPU when `/dev/kvm` is usable, and TCG emulation otherwise
- the image as USB storage on an xHCI controller, the way the laptop boots its stick
- `-no-reboot`, so a triple fault closes QEMU and doesn't loop
- `cargo xtask test` runs the host tests, then the QEMU tests headless with a timeout.
- `cargo xtask ci` runs locally everything CI runs.
- `cargo xtask font` rasterizes `xtask/assets/JetBrainsMono-Regular.ttf` (v2.304, under the SIL Open Font License in `xtask/assets/OFL.txt`) with `fontdue` into one 8-bit coverage bitmap per size in `lib/console/src/font/`, and writes the `font.rs` that embeds them. The TTF, the bitmaps and `font.rs` are committed, so ordinary builds never run the rasterizer. It prints any glyphs it had to clip to their cell.

OVMF comes from the `ovmf-prebuilt` crate at a pinned release, `edk2-stable202605-r1`, which xtask caches in `target/ovmf/`. Each run copies the firmware's variables file to `target/ovmf-vars.fd`, so every boot starts from the same state. xtask never writes to block devices: to test on the laptop, write the image to a USB stick yourself.

## Unsafe code

`thisos-arch-x86_64` is the only crate allowed unsafe code. Every other crate starts with `#![forbid(unsafe_code)]`. This is an attribute and not a workspace lint, because Cargo can't combine a crate's own `[lints]` table with the workspace's. Adding a crate to the exception list is a decision like any other.

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

`clippy::pedantic` is on as a warning group. Four of its lints are allowed: `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc` and `missing_panics_doc`. CI passes `-D warnings`, so pedantic warnings fail the build. `clippy.toml` adds ThisOS to the identifiers that doc comments may write without backticks.
xtask uses clippy's defaults. Formatting is rustfmt's defaults. Identifiers, comments, docs and messages use American spelling, as in "color" and "initialize".

## Tests

- **Host tests.** Generic logic lives in `lib/*` crates and runs under `cargo test` on the host. Every `lib/*` crate also runs its tests under Miri.
- **QEMU tests.** In-kernel tests are `#[test_case]` functions using `custom_test_frameworks`. `cargo test` builds one kernel ELF per test binary. xtask is the cargo runner: it wraps each ELF in an image and boots it in QEMU. Results go to serial and the screen, and the kernel exits QEMU through `isa-debug-exit`.
- **Failures.** A failing test prints its name and panic message and exits QEMU with a failure code. The tests after it in that binary don't run; other binaries still run.
- **Faulting tests.** A test that has to fault, such as a page fault or a stack overflow, goes in a `tests/` file of its own.
- **Shared support.** Test files share their entry point and panic handler through one support module, so that the combined laptop image can include them.
- **Hangs.** At milestone 1, xtask kills QEMU after a fixed time per test binary. From milestone 2, a watchdog on the LAPIC timer stops a hung test and names it.

## Real hardware

Anything that passes in QEMU must also pass on the test laptop described in `docs/design.md`. Laptop runs happen at every milestone and after any change to arch or driver code. A milestone isn't done until it passes there.

To write an image to a USB stick, find the stick's device with `lsblk` (here `/dev/sdX`; writing it erases the stick), unmount it, then:

```
sudo dd if=target/thisos.img of=/dev/sdX bs=4M conv=fsync status=progress
sudo sgdisk -e /dev/sdX
udisksctl power-off -b /dev/sdX
```

`sgdisk -e` moves the image's backup GPT to the end of the stick. The image is only a few MiB, so after `dd` that table sits near the start of the stick. The laptop's firmware doesn't list a stick in that state, and Linux warns about it.

Boot the stick from the laptop's F12 menu, with Secure Boot off.

`cargo xtask build --tests` writes one combined image holding every test that doesn't fault, plus one image per faulting test file. Each image ends on a summary screen listing the tests and their results, with a large PASS or FAIL line, then halts.

## CI

GitHub Actions runs on every push and pull request:

- rustfmt
- clippy with `-D warnings`
- host tests
- Miri
- the QEMU tests
- `cargo deny`

CI caches `target/ovmf/` between runs. Fetching OVMF from GitHub can take minutes, and the cache means CI pays that only when the pinned release changes.

## Dependencies

Dependencies come from crates.io. Each new dependency is reviewed when it is added, and adding one is a decision. Version requirements are caret ranges such as `"0.11"`. `Cargo.lock` is committed, and pins the exact versions. A `cargo update` is a change reviewed like any other. `cargo deny` checks advisories, licenses and sources. Nothing is vendored.
