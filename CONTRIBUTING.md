# Contributing to ThisOS

This file covers how ThisOS code is written, tested and decided. What the system is lives in [docs/design.md](docs/design.md).

## Decisions

Every choice that has a reasonable alternative goes to the maintainer before its code is written. This covers implementation details as well as design. The answers then go to one of three places:

- Answers that change what the system is, or an interface between crates, update `docs/design.md`.
- Answers that are hard to reverse also get a record in `docs/adr/`. These are the syscall ABI, the kernel's object model, IPC semantics and policies like the ones in this file.
- Everything else is stated in the commit message.

An ADR has a status line, a date, and three sections: Context, Decision and Consequences. It is under a page long, and is numbered in sequence (`0005-short-name.md`). A provisional ADR says what will confirm it.

## Work items

Each milestone is a GitHub milestone, and each of its work items is a GitHub issue. An item's issue is opened when work on it starts, and the commit that finishes it closes it. A work item is small enough to review in one sitting, and ends with something that builds or boots.

aarch64 runs behind x86_64 ([0006](docs/adr/0006-aarch64-behind-x86_64.md)). A work item is built and tested for x86_64. When it adds a HAL call, the aarch64 crate gets a stub that panics with `<call> isn't ported to aarch64`, and `cargo xtask ci` has to stay green, which builds and lints aarch64 too. A port to aarch64 is a work item of its own, scheduled through a decision round, and a milestone's closing notes list what aarch64 still lacks.

## Toolchain

The whole repository uses one pinned, dated nightly, set in `rust-toolchain.toml` and bumped by hand. It is the newest nightly that has rustfmt, clippy, Miri, `rust-src` and the `x86_64-unknown-none`, `x86_64-unknown-uefi` and `aarch64-unknown-none-softfloat` targets. The kernel builds for the prebuilt `x86_64-unknown-none` target, and for `aarch64-unknown-none-softfloat` on aarch64.

`rust-toolchain.toml` uses the minimal profile, and lists those components and targets explicitly.

Every crate uses the 2024 edition. Move to the 2027 edition once it is released, expected some time in 2027.

Build profiles:

- `panic = "abort"` in every profile, because the kernel can't unwind.
- Frame pointers are forced for target builds in `.cargo/config.toml`, so a panic can print a backtrace.
- Debug info is on in dev and release.
- Dev builds use `opt-level = 1`. At level 0 the kernel is slow and its stack frames are large.

There are two workspaces:

- The root workspace holds the target crates. Its `.cargo/config.toml` defaults to `x86_64-unknown-none`. Its `default-members` leave out `arch/aarch64`, whose assembly doesn't build for x86_64, so a plain `cargo build` at the root works.
- `xtask/` is its own workspace for host tooling, run as `cargo xtask`. Its own `.cargo/config.toml` sets the target to `x86_64-unknown-linux-gnu`, because cargo also reads the root one and `host-tuple` isn't accepted there; that is what lets rust-analyzer and plain `cargo` commands inside `xtask/` build for the host.

## xtask

- `cargo xtask build` builds the kernel and writes a UEFI GPT disk image to `target/thisos.img` with `bootloader`'s `DiskImageBuilder`, then prints the image's path and its build ID. It uses the dev profile; `--release` builds release. `build`, `run` and `test` take `--arch aarch64`, which builds for aarch64 and writes `target/thisos-aarch64.img` with Limine instead: xtask downloads Limine's binary release 12.9.6 once, checks its SHA-256, caches it in `target/limine/12.9.6/`, and writes the GPT and FAT partition itself with the `gpt` and `fatfs` crates. `flash` is x86_64 only.
  The build ID is `ThisOS <version> <commit> <profile> <time>`, for example `ThisOS 0.0.0 f172dcc-dirty debug 2026-11-04T09:35Z`: the commit is `git describe --always --dirty`, and the time is when xtask built it, in UTC. xtask passes everything after the version to the kernel build as `THISOS_BUILD`. The kernel logs the whole ID as its first line, or `unknown build` in place of the rest when built without xtask. The image holds the ID in `thisos-build.txt` at the root of its FAT partition. `--tests` builds the laptop test image instead (see below), whose ID has `tests` after the profile, as in `ThisOS 0.0.0 3ddbe39 debug tests 2026-11-04T15:00Z`. `cargo xtask test` gives the QEMU tests the same kind of ID.
- `cargo xtask run` builds, then boots the image in QEMU with a window for the framebuffer and serial output on the terminal. QEMU's monitor is on Ctrl-Alt-2 in that window. `--gdb` starts QEMU's gdb stub on TCP port 1234 and starts the CPU paused.

QEMU runs:

- the q35 machine, one CPU and 512 MiB of RAM
- KVM with the host's CPU when `/dev/kvm` is usable, and otherwise TCG emulation of a qemu64 CPU with x2APIC, which the kernel's timer needs, and SMEP and SMAP added
- the image as USB storage on an xHCI controller, the way the laptop boots its stick
- `-no-reboot`, so a triple fault closes QEMU and doesn't loop
- for aarch64, the virt machine with `gic-version=3,acpi=off` instead, a `cortex-a76` under TCG and `-device ramfb`; without `acpi=off`, edk2 hands Limine no device tree
- `cargo xtask flash` builds, then writes the image, or with `--tests` the test image, to the USB stick for a laptop run (see below).
- `cargo xtask test` runs the host tests of every `lib/*` crate, then the QEMU tests with `cargo test -p thisos-kernel`. It is the only way to run the QEMU tests: it sets cargo's runner for the target, as `CARGO_TARGET_X86_64_UNKNOWN_NONE_RUNNER` or `CARGO_TARGET_AARCH64_UNKNOWN_NONE_SOFTFLOAT_RUNNER`, to its own hidden `qemu-test` subcommand, because cargo starts the runner in `kernel/`, where the `xtask` alias's relative manifest path doesn't resolve. A bare `cargo test -p thisos-kernel` runs the kernel ELF on the host, which dies with SIGSEGV.
- `cargo xtask ci` runs locally everything CI runs, in CI's order, and stops at the first failure: the source check for banned patterns that no lint catches (`static mut` and `&'static mut`, see Banned patterns); rustfmt on both workspaces; clippy with `-D warnings` on the root workspace but the aarch64 crate for `x86_64-unknown-none`, on the kernel's tests for the same target, on the kernel and the aarch64 crate for `aarch64-unknown-none-softfloat`, on the kernel's tests for that target, on each `lib/*` crate for the host with its tests, and on xtask with its tests; xtask's host tests; `cargo xtask test` for x86_64; Miri on each `lib/*` crate; `cargo deny` on both workspaces; `cargo xtask build` for both architectures. The aarch64 QEMU tests run only through `cargo xtask test --arch aarch64`. It needs `cargo-deny` installed (`cargo install cargo-deny --locked`) and checks for it first.
- `cargo xtask font` rasterizes `xtask/assets/JetBrainsMono-Regular.ttf` (v2.304, under the SIL Open Font License in `xtask/assets/OFL.txt`) with `fontdue` into one 8-bit coverage bitmap per size in `lib/console/src/font/`, and writes the `font.rs` that embeds them. The TTF, the bitmaps and `font.rs` are committed, so ordinary builds never run the rasterizer. It prints any glyphs it had to clip to their cell.

OVMF comes from the `ovmf-prebuilt` crate at a pinned release, `edk2-stable202605-r1`, which xtask caches in `target/ovmf/`. Each run copies the firmware's variables file to `target/ovmf-vars.fd`, or `target/ovmf-vars-aarch64.fd`, so every boot starts from the same state.

## Unsafe code

`thisos-arch-x86_64` and `thisos-arch-aarch64` are the only crates allowed unsafe code. Every other crate starts with `#![forbid(unsafe_code)]`, except `thisos-frames`, which has `#![deny(unsafe_code)]` and one `#[expect(unsafe_code)]`, on the declaration of `Frame::reclaim`: an unsafe function with no unsafe block, which the arch crate calls to make a `Frame` again from a page-table entry. This is an attribute and not a workspace lint, because Cargo can't combine a crate's own `[lints]` table with the workspace's. Adding a crate to the exception list, or another exception to a crate, is a decision like any other.

Neither `forbid(unsafe_code)` nor clippy's unsafe lints check code that a macro from another crate expands into, and a `macro_rules!` body is only checked where it is expanded. So unsafe code reaches the kernel through `thisos_arch_x86_64::entry!` and the `bootloader_api::entry_point!` it calls, and through `thisos_arch_aarch64::entry!`, whose expansion is a `#[unsafe(no_mangle)]` function that `_start` calls, and no lint checks the SAFETY comment inside either `entry!`. A dependency whose macros a forbidding crate uses is reviewed as unsafe code. A macro in the arch crate that expands to unsafe code follows the rules below as if its expansion were written in the arch crate.

Inside the exception list:

- Every `unsafe` block has a `// SAFETY:` comment saying which invariant holds and why.
- Every `unsafe fn` has a `# Safety` section in its docs.
- Each unsafe block does one unsafe operation.
- Each unsafe operation is wrapped once, in a type whose API is safe and whose invariants are encoded in the type. Examples: physical and virtual address newtypes, a frame that isn't `Copy`, a guard that restores interrupts when dropped, typed MMIO registers.

Tests back up a `// SAFETY:` argument; they don't replace it.

## Banned patterns

These are banned in every crate, with or without unsafe code, and the record of why is [0005](docs/adr/0005-banned-patterns.md). Each says what to do instead, and how it is caught: by a lint, by `banned::check` in `cargo xtask ci`, or only in review. Breaking one is a decision like adding a crate to the unsafe exception list.

Memory and aliasing:

- **`static mut`.** Use a static of an atomic, a write-once cell such as the arch crate's `Once`, a lock, or a wrapper over an `UnsafeCell` that encodes who uses it, as the IST stacks do. Caught by the xtask check.
- **`&'static mut`.** Reach kernel-lifetime memory through a `&'static T` with interior mutability, or through a type that wraps a raw pointer and has a safe API. The one exception is the loader's entry signature, `fn(&'static mut BootInfo) -> !`, which `bootloader_api::entry_point!` fixes: the reference goes no further than the function `entry!` generates and `boot_info`, and never leaves the arch crate. Caught by the xtask check, which allows those two lines, and in review where the lifetime is inferred.
- **References to memory the hardware or another mapping writes**: page tables, whose Accessed and Dirty bits the CPU sets; MMIO; the framebuffer; DMA buffers. Reach it through a raw pointer, one volatile or atomic access at a time, as the framebuffer's `FramebufferPixels` and the arch crate's `paging` module do. The one exception is the slice that `bootloader_api`'s `FrameBuffer::buffer_mut` returns, which is the loader's only way to give the framebuffer's address: `framebuffer()` in the arch crate takes the pointer range from it and never reads or writes through it. Caught in review.
- **Unbounded lifetimes.** A reference made from a raw pointer borrows from the value that owns the memory, or lasts one expression; never `&*ptr` or `slice::from_raw_parts` with a lifetime the caller picks freely. Caught in review.
- **Leaking to get `'static`.** No `Box::leak`, `mem::forget` or `ManuallyDrop` to make a `'static` or skip a destructor; kernel-lifetime values go in statics. Caught by `clippy::mem_forget` and `disallowed-methods` in `clippy.toml`.

Values:

- **`transmute`.** Use `From` and `TryFrom`, `to_le_bytes` and `from_le_bytes`, or a pointer's `cast()`. Caught by `disallowed-methods`.
- **Uninitialized memory.** No `mem::zeroed`, `MaybeUninit::uninit().assume_init()` or `Vec::set_len` over unwritten memory. Use const constructors, and `MaybeUninit::write` before reading. Caught by `disallowed-methods` and clippy's default `uninit_assumed_init` and `uninit_vec`.
- **Integers turned into pointers.** Pointers keep their provenance through `addr`, `with_addr` and `map_addr`. An integer becomes a pointer, through exposed provenance or the `x86_64` crate's `VirtAddr::as_ptr`, only in an arch crate, and only for an address that reaches it as an integer from the hardware or the loader, or that the arch crate maps itself: a register such as `rbp`, `x29`, CR3 or TTBR1, a `BootInfo` field, a physical address through the physical-memory mapping, device registers the aarch64 crate maps there, or a page the arch crate maps in the reserved level 4 entry (level 0 on aarch64), such as the heap's base. On aarch64 every such conversion goes through the crate's one `ptr_at`. The SAFETY comment or a comment at the conversion says which. `clippy::as_conversions` already rules out `as` casts; `disallowed-methods` catches `ptr::with_exposed_provenance`, and review catches `as_ptr`.
- **Unchecked operations** such as `get_unchecked`, `unreachable_unchecked`, `unwrap_unchecked` and `str::from_utf8_unchecked`. Use the checked one; using an unchecked one needs a measurement that the check matters, and a decision. Caught by `disallowed-methods`.

Concurrency, handlers and escape hatches:

- **A lock shared with a handler.** A trap or interrupt handler never takes a lock that ordinary code holds while traps or interrupts can arrive, because the handler then spins forever. The panic path's `try_lock` is how the trap report avoids it, and the logger holds its lock only with interrupts off, so that interrupt handlers can log. Caught in review.
- **`unsafe impl Send` or `Sync` on a type that doesn't encode the invariant.** Only a type that wraps one resource and says in its SAFETY comment why sharing or moving it is sound, like `Once`, the IST `Stack` and `FramebufferPixels`; never a plain data struct, to get it into a static. Caught in review.
- **`#[allow]`.** Silence a lint with `#[expect(lint, reason = "...")]`, so that an exception nobody needs any more fails the build. Caught by `clippy::allow_attributes` and `clippy::allow_attributes_without_reason`.
- **Unsafe attributes outside the arch crates**: `no_mangle`, `export_name`, `link_section`. They appear only in the arch crates or in their macros' expansions. Caught by `forbid(unsafe_code)`, which counts them as unsafe code, except inside other crates' macro expansions, as described above.

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
- `clippy::allow_attributes`, `clippy::allow_attributes_without_reason`, `clippy::mem_forget` and `clippy::disallowed_methods`, for the banned patterns above. `clippy.toml` lists the disallowed methods with what to use instead, and applies to xtask too

`clippy::pedantic` is on as a warning group. Four of its lints are allowed: `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc` and `missing_panics_doc`. CI passes `-D warnings`, so pedantic warnings fail the build. Test code is linted too: the kernel's `cfg(test)` code, each `lib/*` crate's tests and xtask's tests. `clippy.toml` adds ThisOS to the identifiers that doc comments may write without backticks.
xtask uses clippy's defaults, plus its own `[lints.clippy]` table that denies the banned-pattern lints above except `disallowed_methods`, which `-D warnings` already makes an error there. Formatting is rustfmt's defaults. Identifiers, comments, docs and messages use American spelling, as in "color" and "initialize".

## Tests

- **Host tests.** Generic logic lives in `lib/*` crates and runs under `cargo test` on the host. Every `lib/*` crate also runs its tests under Miri.
- **QEMU tests.** In-kernel tests are `#[test_case]` functions using `custom_test_frameworks`. `cargo test` builds one kernel ELF per test binary. xtask is the cargo runner: it writes each ELF into an image next to it, `<elf>.img`, and boots it headless in QEMU with serial on standard output. The kernel logs `test <name>` before each test and `<n> tests passed` after the last, to serial and the screen, unless a test passes by panicking, which ends the run before that line. It ends on a summary: `summary:`, then a line per test saying `ok`, `FAILED` or `not run`, then PASS or FAIL, four times the font's size on the screen. A panic outside any test puts `panic before the tests started` in place of the test lines, or `panic after the last test` after them. Then it exits QEMU through `isa-debug-exit` at port 0xf4: 0x10 for success, which QEMU turns into exit status 33, and 0x11 for failure, status 35. Only 33 is a pass, because `-no-reboot` turns a triple fault into status 0. The kernel writes the port only when CPUID reports a hypervisor, which QEMU does with KVM and with TCG; on the laptop, where nobody knows what 0xf4 decodes to, it halts after the summary. On aarch64 the kernel exits through semihosting's `SYS_EXIT_EXTENDED` with the same two statuses, which xtask's `-semihosting-config enable=on,target=native` turns on for test runs, and only when the device tree's root is QEMU's virt machine; aarch64 has no serial port, so its test output is on the screen alone. `thisos-kernel` is a library with a thin binary, and every kernel test lives in the library under `cfg(test)`, so there is one kernel test binary.
- **Failures.** A failing test's panic report is followed by `test <name> failed` and the summary, and the kernel exits QEMU with failure. The tests after it don't run.
- **Panicking tests.** A test that has to panic is a `#[test_case]` const of `testing::ExpectPanic`, naming part of the message it expects. A panic ends the run, so the runner runs it after every other test and refuses more than one. The panic handler counts it as passed only if the message contains that part, and otherwise as failed. A test that returns without panicking fails. `logger::tests` has the one that exists, which panics inside the logger to check that the report reaches the panic sinks.
- **Trapping tests.** A test that has to trap is an ordinary `#[test_case]` that runs a `thisos_hal::Probe` through `testing::expect_trap(|trap| ..., probe)`, or `testing::expect_timeout(seconds, probe)` for the watchdog. The probe is code the arch crate writes in assembly: reading the guard page, pushing until the stack overflows into it, which runs with interrupts off so that no interrupt handler overflows the stack in its place, spinning, writing back a read-only byte, calling a `ret` in a no-execute page, or reading or calling a fixed user address, which SMAP and SMEP have to stop, or PAN and PXN on aarch64. The last four use targets the arch crate owns, which `Platform::probe_target` names, so kernel code can't aim a probe at other memory. One more probe opens user access, as a user copy does, and spins, so that a test can check that the timer handler which stops it finds access blocked. `Cpu::catch` saves the registers before it runs the probe. When the kernel's trap handler sees the trap the test expects, `TrapFrame::abandon` points the frame back at them, so the CPU returns into the test, which passes, and the run goes on; that works after a double fault too. Any other trap fails the test as usual. The probes are assembly because returning this way skips the frames in between without running their destructors, which Rust allows only for frames that have none.
- **User-mode tests.** A test that needs code in ring 3 maps one of the arch crate's `thisos_hal::UserProgram`s, which `Platform::write_user_program` writes into a frame, in a new space with a stack page, and starts it with `thread::spawn_user`. Each program ends by the exit syscall or by a trap that kills its thread, and `user::tests::run` waits for that, checks that the thread's stack and every frame of its space came back, and returns how it ended.
- **Hangs.** A watchdog on the LAPIC timer stops a test that runs longer than 10 s. Its report reads like a trap's: `test <name> timed out after 10 s`, the interrupted code's registers and a backtrace from its instruction, then the summary with that test FAILED. Ticks that come while interrupts are off count as one, so a test that logs a lot gets somewhat more than 10 s. xtask still kills QEMU after 60 s per test binary and fails it, for a hang with interrupts off, which the watchdog can't see.

## Real hardware

Anything that passes in QEMU must also pass on the test laptop described in `docs/design.md`. Laptop runs happen at every milestone and after any change to arch or driver code. A milestone isn't done until it passes there. Each run's result goes in a comment on its work item's issue: the build ID the screen shows first, PASS or FAIL with the test that failed, and anything the run showed about the hardware.

`cargo xtask flash` writes the image to a USB stick, erasing it. It takes `--release`, `--tests` and `--device PATH`. It runs as you, and runs `dd` and `sgdisk` through `sudo`. It:

- only considers whole disks on USB that are removable or hotplug, and never the internal disk. With one such disk it uses it; with several, `--device` picks one.
- refuses a disk with no `/dev/disk/by-id` link, and names the disk by that link in every command after choosing it. A stick swapped while flash waits for an answer or a password takes its link with it, and `dd` runs with `conv=nocreat`, so it fails instead of writing the image to a new file at the link's path, and no other disk is written.
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

For aarch64 the ELF is `target/aarch64-unknown-none-softfloat/debug/thisos-kernel`.

`llvm-addr2line` takes the same arguments.

`cargo xtask build --tests` writes the kernel library's test binary, which holds every kernel test, into `target/thisos-tests.img`, and `cargo xtask flash --tests` writes that to the stick, so a full laptop run is one flash and one boot. xtask builds it with `cargo test --no-run --lib` and finds it in cargo's JSON messages. The image ends on the summary screen described under Tests, with the panic report above it when a test fails, then halts.

## CI

GitHub Actions runs on every push and pull request:

- rustfmt
- clippy with `-D warnings`
- host tests
- Miri
- the x86_64 QEMU tests, under TCG
- `cargo deny`
- an image build

`.github/workflows/ci.yml` has one job on `ubuntu-26.04` that runs `cargo xtask ci`, so CI and a local run do the same checks. It installs the toolchain from `rust-toolchain.toml` with the runner's rustup, `cargo-deny` 0.20.2 as a prebuilt binary through `taiki-e/install-action`, and QEMU from Ubuntu's `qemu-system-x86` package, which the runner image lacks. Every action is pinned to a full commit SHA, with its tag in a comment. The workflow only reads the repository, and a new push cancels a run still going on the same ref.

CI caches the cargo registry and both target directories, keyed on `rust-toolchain.toml` and both `Cargo.lock` files. The cache holds `target/ovmf/` too: fetching OVMF from GitHub can take minutes, and the cache means CI pays that only when the pinned release changes. The `cargo xtask ci` step times out after 15 minutes on an exact cache hit and 45 otherwise, and the job after 60, which also bounds the network steps before it.

## Dependencies

Dependencies come from crates.io. Each new dependency is reviewed when it is added, and adding one is a decision. Version requirements are caret ranges such as `"0.11"`. `Cargo.lock` is committed, and pins the exact versions. A `cargo update` is a change reviewed like any other. `cargo deny` checks advisories, licenses and sources against `deny.toml`, which both workspaces share. It allows exactly the licenses the dependencies use today, so a dependency with a new license fails until the license is added there, as part of that dependency's review. Unmaintained crates fail like other advisories; an ignored advisory is listed in `deny.toml` with its reason. Crates come from crates.io only, and duplicate versions are a warning. Nothing is vendored.
