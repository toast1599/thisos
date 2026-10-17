# 02. Trusted base and build pipeline

- **Day:** 2 (Thu 2026-10-15)
- **Status:** In progress

## Covers

What the system has to trust, how programs are built and checked before they run, and which of those checks still matter now that the hardware isolates programs ([0014](../decisions/0014-microkernel.md)).

## Decided

- [0005](../decisions/0005-unsafe-code-policy.md) Unsafe code exists only in one closed, audited trusted set (superseded by [0017](../decisions/0017-unsafe-code-kernel-and-services.md))
- [0012](../decisions/0012-no-memory-cap.md) No fixed limit on program memory
- [0014](../decisions/0014-microkernel.md) Microkernel: a minimal, audited kernel in kernel mode; everything else in user mode (amended by [0082](../decisions/0082-corrections.md) and [0169](../decisions/0169-iommu-service.md))
- [0017](../decisions/0017-unsafe-code-kernel-and-services.md) Unsafe code: one closed, audited set for the kernel and the OS's own services
- [0037](../decisions/0037-kernel-toolchain.md) The kernel is built with one pinned, audited toolchain
- [0038](../decisions/0038-elf-container.md) Programs ship as ELF, used only as a file container
- [0039](../decisions/0039-native-per-arch.md) Programs ship as native code, built separately for each architecture
- [0040](../decisions/0040-signed-programs-owner-policy.md) Programs are signed, and the device owner's policy decides which signers may run
- [0041](../decisions/0041-immutable-packages.md) Installed programs are immutable packages named by their content hash
- [0042](../decisions/0042-updates.md) The system updates atomically as one image; third-party programs update package by package
- [0043](../decisions/0043-executable-memory-capability.md) Making memory executable at run time needs a capability
- [0044](../decisions/0044-system-image-is-os-shipped.md) Everything in the system image counts as shipped by the OS

## Key finding: the Rust compiler is not itself a soundness proof

Measured on 2026-10-14 with rustc 1.100.0-nightly (2026-09-01) and rustc 1.99.0 stable (2026-09-28), edition 2024:

- A program with `#![forbid(unsafe_code)]` and no `unsafe` anywhere compiled on **both** toolchains and, at runtime, read heap memory that had already been freed (a use-after-free). It relied only on safe references.
- This is rust-lang/rust#25860 ("Implied bounds on nested references + variance = soundness hole"), open since 2015 and still open on 2026-10-14. The higher-ranked `fn`-pointer form that was tested is the shape the `cve-rs` crate uses. The plain form from the issue is now rejected; the higher-ranked form is not.
- As of 2026-10-14, rust-lang/rust had 141 open issues labelled `I-unsound`.

**Implication for this OS.** "The compiler accepted the program" is not equivalent to "the program cannot violate memory safety." Under [0002](../decisions/0002-kernel-mode-verified-execution.md), a verifier that trusts `rustc` acceptance inherits every one of these holes as a full kernel-mode compromise. This is the central tension the verification design has to resolve. It does not mean the goal is unreachable; it means the trusted check cannot be "rustc exited 0." Directions to weigh on Day 2:

- Pin a known-good toolchain and maintain a denylist of patterns that trigger known holes (weakest; a denylist never proves completeness).
- Add an independent check on the compiler's *output* rather than trusting its *acceptance*, so a `rustc` front-end bug cannot by itself pass bad code (the Verve/typed-output approach).
- Treat the open soundness bugs as a tracked, finite list the OS commits to driving to closure or blocking individually, and record that commitment.

**Since [0014](../decisions/0014-microkernel.md).** This now matters only for code in kernel mode, which is the kernel itself. A hole that a user-mode program triggers can only break that program.

## Measurements (2026-10-15)

All measurements were made on the development machine: an Intel Core Ultra 5 125H, pinned to one performance core, with rustc 1.100.0-nightly (2026-09-01). The benchmark programs are not part of the repository. These measurements compared the kernel-mode designs that led to [0014](../decisions/0014-microkernel.md). The costs of entering the kernel still apply to the microkernel.

### Where the soundness holes are

As of 2026-10-15, rust-lang/rust had 143 open issues labelled `I-unsound`. Of these:

- 40 need nightly features, which [0005](../decisions/0005-unsafe-code-policy.md)'s flags block.
- 26 affect only other platforms.
- The other 77 apply to this OS:
  - **39** are in the front end: the type system, trait solving and borrow checking. #25860 is one of them.
  - **22** come after type checking: MIR optimizations, LLVM and code generation. About half need `unsafe` or special compiler flags, which leaves about 12 reachable from safe code.
  - **7** are in std, and **9** are elsewhere.
- 31 of those 77 were opened in 2026.

The issues were sorted by their labels, and the titles were checked by hand.

For #25860, two fix attempts were closed without merging, and a third (#160461) has been open since August 2026. LLVM had 572 open issues labelled `miscompilation`, and 514 were filed in 2026.

### Cost of isolating programs in software

The same source for 17 workloads was built natively and for `wasm32-wasip1` with 128-bit SIMD. The workloads cover hashing, floating point, sorting, hash maps, allocation, an interpreter, dynamic dispatch, memory copies, JSON, regex and compression. Each workload times only its own work. There were six interleaved rounds, and the results are geometric means relative to native:

| Configuration | Overhead |
|---|---|
| WebAssembly compiled by LLVM (Wasmer 7.5.0), guard pages | +12% |
| WebAssembly compiled by Cranelift (Wasmtime 49.0.2), guard pages | +26% |
| Cranelift, explicit bounds check on every access | +41% |
| Cranelift, bounds checks plus Spectre masking | +62% |

The overall result varied by about 6 points between rounds.

- **Guard pages** only work with 32-bit addresses, which [0010](../decisions/0010-64-bit-only.md) and [0012](../decisions/0012-no-memory-cap.md) rule out. 64-bit memory needs explicit bounds checks.
- **Compiler quality:** Cranelift accounts for about 12 points over LLVM.
- **SIMD width:** WebAssembly only has 128-bit SIMD.
- **LFI** isolates native code instead of WebAssembly. Its published figures on x86-64 are +7.1–7.9% for full isolation, +3.8–5.3% when only writes and jumps are checked, and +3.2–4.3% for jumps only. On AArch64, full isolation costs +6.4–7.3%. As published, it confines each program to a 4 GiB region.

### Cost of entering the kernel

These were measured inside a KVM guest, by a UEFI program running in kernel mode. The timestamp counter ran at 2.995 GHz.

| Operation | Cycles | Time |
|---|---|---|
| Call plus two page-table switches with PCID and no flush: the cheapest kernel call under D | 343 | 114 ns |
| `syscall` and `sysretq` round trip from user mode: the cheapest traditional kernel call | 109 | 36 ns |
| IBPB (branch-predictor flush) | 545 | 182 ns |

- **Without PCID,** each switch flushes the TLB. Refilling 64 pages after the switches cost another 273 cycles.
- **On the host,** a full Linux system call (`getppid`) took 83 ns, and an indirect function call took 1 ns.
- **The CPU isn't affected by Meltdown,** so Linux keeps the kernel mapped in every process.
- **Its branch-predictor defences** (Enhanced IBRS, BHI_DIS_S) separate user mode from kernel mode.

### Cost of hardening a shared address space against Spectre

This used the same workloads built natively, with std rebuilt from source with the same flags:

- **Speculative load hardening** (Spectre variant 1) cost +44% over 16 workloads. Individual rounds ranged from +27% to +79%.
- **Adding retpolines** (Spectre variant 2) brought it to +69%. That figure is inflated by indirect calls into the C library, which a statically linked OS wouldn't have.
- **One workload was left out.** With speculative load hardening alone, the interpreter workload crashed: an out-of-bounds write in safe code that runs correctly without the flag. It appears to be an LLVM miscompilation, but it hasn't been reduced to a minimal case.

### Options compared

- **A. Trusted compiler, signed builds.**
  - Costs nothing at run time.
  - The trusted base is rustc, LLVM and std, and a hostile author can use any of the reachable holes above.
  - The device can't check anything itself, so a central build service has to receive every program's source.
- **B. Independent check of the compiled output.** It's done in steps:
  - **B1** re-checks rustc's MIR at the build service. It removes the front end from the trusted base, but LLVM stays.
  - **B2** moves the check to the device.
  - **B3** checks typed machine code. It's research with no precedent for Rust, for three reasons: Rust frees memory without a garbage collector, LLVM no longer tracks pointer types, and std's unsafe code is inlined into every program.
- **C. Bounds-check every access.** +41% to +62% without a memory cap.
- **D. Separate checked address spaces in kernel mode.**
  - No cost per access, no memory cap, and the compiler isn't trusted.
  - Kernel calls cost two page-table switches.
  - Hardware Spectre defences that rely on the user/kernel split don't apply.

Neither A nor B stops speculative reads inside one shared address space. Once address spaces are separate, kernel mode no longer saves anything, and D costs more than the user-mode isolation that 0002 rejected. [0013](../decisions/0013-isolation-domains.md) proposed combining A/B for the system's own components with D for every other program. We chose a microkernel instead ([0014](../decisions/0014-microkernel.md)), which makes program isolation independent of all four.

## Open questions

- **Signing** ([0040](../decisions/0040-signed-programs-owner-policy.md)): the default owner policy, how signing keys are managed, and who signs the system image and the boot loader. At boot, the boot loader checks the kernel and the ramdisk, and the package store checks the rest of the image [0156](../decisions/0156-boot-loader-checks-image.md). Whether the loader or a separate service checks programs installed later is open.
- **Program sections** ([0038](../decisions/0038-elf-container.md)): the ELF sections carry only the program's declarations [0147](../decisions/0147-elf-declarations-only.md). Programs are built position-independent [0070](../decisions/0070-position-independent-programs.md). Which relocations they may carry is open, and so is whether a program can ask for its own image to be placed at low addresses [0078](../decisions/0078-low-addresses-on-request.md).
- **Packages** ([0041](../decisions/0041-immutable-packages.md)): the hash function, the package layout and the service that stores packages. The package around the image carries the signature and the package metadata [0147](../decisions/0147-elf-declarations-only.md). A list in the ramdisk names the system image's other packages by hash, and the boot loader's check covers it ([0155](../decisions/0155-ramdisk-reaches-storage.md), [0156](../decisions/0156-boot-loader-checks-image.md)).
- **Updating the boot loader** ([0158](../decisions/0158-image-try-count.md)). The boot loader sits outside the system images it chooses between, so their rollback doesn't cover it. How is it updated?
- **Outdated std in third-party programs** ([0042](../decisions/0042-updates.md)). A std fix reaches a third-party program only when its publisher rebuilds it. What does the OS do about programs built with a flawed std?
- **The audited set** ([0017](../decisions/0017-unsafe-code-kernel-and-services.md)). List the kernel's unsafe code, the boot loader's [0149](../decisions/0149-bootloader-crate-uefi.md), and the shared wrappers the OS's own services use, with an audit record for each part.
