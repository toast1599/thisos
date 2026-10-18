# 02. Trusted base and build pipeline

- **Day:** 2 (Thu 2026-10-15)
- **Status:** Frozen

## Covers

This is about what the system has to trust, and how programs get built and checked before they run. Now that the hardware isolates programs ([0014](../decisions/0014-microkernel.md)), it also covers which of those checks still matter.

## Decided

- [0005](../decisions/0005-unsafe-code-policy.md) Unsafe code exists only in one closed, audited trusted set (superseded by [0017](../decisions/0017-unsafe-code-kernel-and-services.md))
- [0012](../decisions/0012-no-memory-cap.md) No fixed limit on program memory
- [0013](../decisions/0013-isolation-domains.md) Isolation domains: one shared system domain, one checked address space per program (superseded by [0014](../decisions/0014-microkernel.md), never accepted)
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

On the toolchain and the build, from [07-engineering](07-engineering.md):

- [0187](../decisions/0187-pinned-stable-toolchain.md) The toolchain is a pinned stable Rust release, and std's platform layer is a patch over its library
- [0188](../decisions/0188-upgrade-every-second-release.md) The toolchain is upgraded every second stable release
- [0191](../decisions/0191-audited-list-and-wrapper.md) A list of audited crates, enforced by a compiler wrapper, keeps unsafe code in the audited set
- [0192](../decisions/0192-build-scripts-reviewed.md) Build scripts and procedural macros run only in reviewed crates, and never link native code
- [0193](../decisions/0193-vendored-reviewed-crates.md) Third-party crates in the OS's own code are vendored and reviewed (amended by [0243](../decisions/0243-audit-records-and-crate-reviews.md))
- [0194](../decisions/0194-reproducible-builds.md) The system image and the boot loader build reproducibly

Signing and packages were settled in the final review:

- [0202](../decisions/0202-offline-attackers.md) Attackers who get the machine while it's off are in scope
- [0203](../decisions/0203-package-format.md) A package is a signed manifest and files named by SHA-256 Merkle roots
- [0204](../decisions/0204-hybrid-signatures.md) Signatures pair Ed25519 with ML-DSA-65
- [0205](../decisions/0205-root-and-working-keys.md) An offline root key names the working keys, and the boot loader carries only the root
- [0206](../decisions/0206-owner-approves-signers.md) A program runs if the system image's signer or a signer the owner approved signed it
- [0207](../decisions/0207-install-and-start-checks.md) The package store checks packages at install and after each boot, and the loader checks the policy at each start
- [0208](../decisions/0208-boot-loader-beside-old.md) A new boot loader is installed beside the previous one, which the firmware falls back to (amended by [0223](../decisions/0223-qemu-and-laptop-checkpoints.md))
- [0209](../decisions/0209-no-rollback-protection.md) Older signed images still boot, and rollback protection is later work
- [0210](../decisions/0210-std-version-declared.md) Programs declare the std they were built with, and the owner can refuse flawed ones
- [0211](../decisions/0211-relative-relocations.md) Programs carry only relative relocations
- [0212](../decisions/0212-low-image-declaration.md) A program can declare the highest address for its image

Day 8 ([08-roadmap](08-roadmap.md)) brought the records on reviews, audits and tests:

- [0235](../decisions/0235-kinds-of-tests.md) Every change runs host, QEMU, check and attack tests, and Miri runs on a test-only nightly
- [0241](../decisions/0241-reproducibility-checks.md) The image and the boot loader are built twice in CI, and rebuilt elsewhere before signing
- [0242](../decisions/0242-review-tiers.md) Changes are reviewed in three tiers, by role
- [0243](../decisions/0243-audit-records-and-crate-reviews.md) Each audited crate has a record beside the list, and cargo-vet records crate reviews
- [0244](../decisions/0244-imported-crate-reviews.md) Crate reviews may be imported from named organizations, outside the audited set

## Key finding: the Rust compiler is not itself a soundness proof

We measured this on 2026-10-14, with rustc 1.100.0-nightly (2026-09-01) and rustc 1.99.0 stable (2026-09-28), edition 2024. A program with `#![forbid(unsafe_code)]`, and no `unsafe` anywhere in it, compiled on both toolchains. At run time it read heap memory that had already been freed (a use-after-free), relying only on safe references.

The bug is rust-lang/rust#25860 ("Implied bounds on nested references + variance = soundness hole"). It has been open since 2015, and on 2026-10-14 it was still open. What we tested was its higher-ranked `fn`-pointer form, which is the shape the `cve-rs` crate uses. The compiler now rejects the plain form from the issue, but not the higher-ranked one. And it isn't a lone case: as of 2026-10-14, rust-lang/rust had 141 open issues labelled `I-unsound`.

So what does that mean for this OS? "The compiler accepted the program" is not the same thing as "the program cannot violate memory safety." Under [0002](../decisions/0002-kernel-mode-verified-execution.md), a verifier that trusts `rustc` acceptance inherits every one of these holes, and each of them is a full kernel-mode compromise. That's the central tension the verification design has to resolve. It doesn't put the goal out of reach. What it does mean is that the trusted check cannot be "rustc exited 0." These were the directions to weigh on Day 2:

- Pin a known-good toolchain, and keep a denylist of patterns that trigger known holes. It's the weakest of the three, because a denylist never proves completeness.
- Check the compiler's *output* independently instead of trusting its *acceptance*, so that a `rustc` front-end bug cannot pass bad code on its own. This is the Verve/typed-output approach.
- Treat the open soundness bugs as a tracked, finite list, commit to driving each one to closure or blocking it individually, and record that commitment.

[0014](../decisions/0014-microkernel.md) changed how much this matters. Now it matters only for code in kernel mode, which means the kernel itself. If a user-mode program triggers one of these holes, the hole can only break that program.

## Measurements (2026-10-15)

We made all of these measurements on the development machine, an Intel Core Ultra 5 125H, pinned to one performance core and using rustc 1.100.0-nightly (2026-09-01). The benchmark programs aren't in the repository. What they compared were the kernel-mode designs that led to [0014](../decisions/0014-microkernel.md), but the costs of entering the kernel still apply to the microkernel.

### Where the soundness holes are

As of 2026-10-15, rust-lang/rust had 143 open issues labelled `I-unsound`. Of these, 40 need nightly features, which [0005](../decisions/0005-unsafe-code-policy.md)'s flags block, and 26 affect only other platforms. The other 77 apply to this OS:

- **39** in the front end, meaning the type system, trait solving and borrow checking. #25860 is among them.
- **22** after type checking, in MIR optimizations, LLVM and code generation. About half of these need `unsafe` or special compiler flags, which leaves about 12 that safe code can reach.
- **7** in std, and **9** elsewhere.

Of those 77, 31 were opened in 2026. We sorted the issues by their labels and checked the titles by hand.

Two attempts to fix #25860 were closed without merging, and a third (#160461) has been open since August 2026. LLVM had 572 open issues labelled `miscompilation`, and 514 were filed in 2026.

### Cost of isolating programs in software

We built the same source for 17 workloads natively and for `wasm32-wasip1` with 128-bit SIMD. Between them the workloads cover hashing, floating point, sorting, hash maps, allocation, an interpreter, dynamic dispatch, memory copies, JSON, regex and compression, and each one times only its own work. There were six interleaved rounds. The results are geometric means relative to native:

| Configuration | Overhead |
|---|---|
| WebAssembly compiled by LLVM (Wasmer 7.5.0), guard pages | +12% |
| WebAssembly compiled by Cranelift (Wasmtime 49.0.2), guard pages | +26% |
| Cranelift, explicit bounds check on every access | +41% |
| Cranelift, bounds checks plus Spectre masking | +62% |

Between rounds, the overall result moved by about 6 points.

Some context for these numbers. Guard pages work only with 32-bit addresses, and [0010](../decisions/0010-64-bit-only.md) and [0012](../decisions/0012-no-memory-cap.md) rule those out, so 64-bit memory needs explicit bounds checks. Compiler quality counts as well: Cranelift accounts for about 12 points over LLVM. SIMD width plays a part too, since WebAssembly only has 128-bit SIMD.

LFI takes a different route and isolates native code instead of WebAssembly. On x86-64 its published figures are +7.1–7.9% for full isolation, +3.8–5.3% when only writes and jumps are checked, and +3.2–4.3% for jumps only. On AArch64, full isolation costs +6.4–7.3%. As published, though, it confines each program to a 4 GiB region.

### Cost of entering the kernel

A UEFI program running in kernel mode took these measurements inside a KVM guest. The timestamp counter ran at 2.995 GHz.

| Operation | Cycles | Time |
|---|---|---|
| Call plus two page-table switches with PCID and no flush: the cheapest kernel call under D | 343 | 114 ns |
| `syscall` and `sysretq` round trip from user mode: the cheapest traditional kernel call | 109 | 36 ns |
| IBPB (branch-predictor flush) | 545 | 182 ns |

Without PCID, each switch flushes the TLB, and refilling 64 pages after the switches cost another 273 cycles. For comparison, on the host a full Linux system call (`getppid`) took 83 ns and an indirect function call took 1 ns.

The CPU isn't affected by Meltdown, so Linux keeps the kernel mapped in every process. Its branch-predictor defences (Enhanced IBRS, BHI_DIS_S) separate user mode from kernel mode.

### Cost of hardening a shared address space against Spectre

For this we used the same workloads built natively, with std rebuilt from source using the same flags.

Speculative load hardening, against Spectre variant 1, cost +44% over 16 workloads, and individual rounds ranged from +27% to +79%. Adding retpolines for Spectre variant 2 brought it to +69%. That figure is inflated, though, by indirect calls into the C library, which a statically linked OS wouldn't have.

One workload was left out. With speculative load hardening alone, the interpreter workload crashed with an out-of-bounds write in safe code, code that runs correctly without the flag. It looks like an LLVM miscompilation, but nobody has reduced it to a minimal case.

### Options compared

- **A. Trusted compiler, signed builds.** It costs nothing at run time. The trusted base is rustc, LLVM and std, though, and a hostile author can use any of the reachable holes above. And since the device can't check anything itself, a central build service has to receive every program's source.
- **B. Independent check of the compiled output,** done in steps. B1 re-checks rustc's MIR at the build service, which takes the front end out of the trusted base but leaves LLVM in it. B2 moves the check to the device. B3 checks typed machine code, and that's research with no precedent for Rust, for three reasons: Rust frees memory without a garbage collector, LLVM no longer tracks pointer types, and std's unsafe code is inlined into every program.
- **C. Bounds-check every access.** It costs +41% to +62% without a memory cap.
- **D. Separate checked address spaces in kernel mode.** There's no cost per access and no memory cap, and the compiler isn't trusted. On the other hand, kernel calls cost two page-table switches, and hardware Spectre defences that rely on the user/kernel split don't apply.

Neither A nor B stops speculative reads inside one shared address space. Once the address spaces are separate, kernel mode no longer saves anything, and D ends up costing more than the user-mode isolation that 0002 rejected. [0013](../decisions/0013-isolation-domains.md) proposed a mix: A/B for the system's own components, and D for every other program. We chose a microkernel instead ([0014](../decisions/0014-microkernel.md)), and that makes program isolation independent of all four.

## Open questions

None are left in this document, because the final review ([07-engineering](07-engineering.md)) settled them all:

- Signing ([0040](../decisions/0040-signed-programs-owner-policy.md)) uses signatures that pair Ed25519 with ML-DSA-65 ([0204](../decisions/0204-hybrid-signatures.md)). An offline root key names the working keys, and the boot loader carries only the root's public key ([0205](../decisions/0205-root-and-working-keys.md)). A program runs if it was signed by the system image's signer or by a signer the owner approved ([0206](../decisions/0206-owner-approves-signers.md)). The package store checks packages at install and after each boot, and the loader checks the owner's policy at each start ([0207](../decisions/0207-install-and-start-checks.md)).
- For program sections ([0038](../decisions/0038-elf-container.md)), programs carry only relative relocations ([0211](../decisions/0211-relative-relocations.md)), and a program can declare the highest address for its image ([0212](../decisions/0212-low-image-declaration.md)).
- A package ([0041](../decisions/0041-immutable-packages.md)) is a signed manifest plus files named by SHA-256 Merkle roots ([0203](../decisions/0203-package-format.md)). The package store is a service of its own, and it lives in the ramdisk ([0155](../decisions/0155-ramdisk-reaches-storage.md)).
- To update the boot loader ([0158](../decisions/0158-image-try-count.md)), we install the new one beside the previous one, which the firmware falls back to ([0208](../decisions/0208-boot-loader-beside-old.md)). Older signed images and boot loaders still boot, and rollback protection is later work ([0209](../decisions/0209-no-rollback-protection.md)).
- Third-party programs might carry an outdated std ([0042](../decisions/0042-updates.md)), so programs declare the std they were built with, and the owner can refuse flawed ones ([0210](../decisions/0210-std-version-declared.md)).
- For the audited set ([0017](../decisions/0017-unsafe-code-kernel-and-services.md)), a list in the repository names the crates that may contain unsafe code, each with its audit record, and a compiler wrapper enforces it ([0191](../decisions/0191-audited-list-and-wrapper.md)). We fill the list in as the parts are written, and each crate's audit record takes the form that [0243](../decisions/0243-audit-records-and-crate-reviews.md) sets.
