# 0014. Microkernel: a minimal, audited kernel in kernel mode; everything else in user mode

- **Status:** Accepted, amended by [0082](0082-corrections.md) and [0169](0169-iommu-service.md)
- **Date:** 2026-10-15

## Context

[0002](0002-kernel-mode-verified-execution.md) put every program in kernel mode and isolated programs by verification. Day 2's comparison and measurements ([02-verification](../design/02-verification.md)) found two problems with that approach:

- **One shared address space** makes isolation depend on rustc and LLVM, and 77 open soundness holes apply to this OS. Language safety also doesn't stop one program reading another's memory speculatively. Software hardening against even one Spectre variant cost +44%.
- **Separate checked address spaces in kernel mode** ([0013](0013-isolation-domains.md)) keep programs in kernel mode but give each its own address space. That rebuilds the user/kernel split in software:
  - it needs a code checker for each architecture;
  - its kernel calls cost 343 cycles, against 109 for a hardware user/kernel round trip;
  - it loses the hardware Spectre defences built around that split.

Week 1 is the experimental phase, and the architect chose to move to a traditional microkernel.

Prior art:
- seL4: on the order of 10,000 lines, with a machine-checked proof of correctness.
- The L4 family.
- QNX: in production use for decades.
- MINIX 3: drivers run in user mode and are restarted when they fail.
- Zircon: Fuchsia's kernel.
- Written in Rust: Redox and Hubris.

## Options

1. Keep 0002: every program in kernel mode, in one shared address space, isolated by Rust.
2. Kernel mode everywhere, with a shared system domain and a checked address space for each program ([0013](0013-isolation-domains.md)).
3. A monolithic kernel: programs isolated by hardware, with drivers, filesystems and the network stack inside the kernel.
4. A microkernel: programs isolated by hardware, with only a minimal kernel in kernel mode and everything else in user mode.

## Decision

Option 4.

- **Kernel mode is for the kernel only.** The kernel runs in kernel mode: ring 0 on x86_64, EL1 on AArch64. Everything else runs in user mode: ring 3 on x86_64, EL0 on AArch64. That includes drivers, filesystems, the network stack, system services and applications.
- **The kernel is minimal.** It contains only what has to run in kernel mode. It is kept very small, and it is heavily audited and tested. Everything else is built on top of it.
- **The hardware isolates programs,** through privilege levels and a separate address space for each program.

## Consequences

- **0002 and 0013 are superseded.** The aim of a trusted base as small as possible carries over. The trusted base is now the kernel, the boot path and the hardware.
- **The compiler no longer isolates programs.** A rustc or LLVM bug can only break the program it miscompiled. It still matters for the kernel, which is one more reason to keep the kernel small.
- **Hardware defends against speculative reads between programs.** Separate address spaces and the user/kernel split are what the hardware defences were built for (on the development machine, Enhanced IBRS and BHI_DIS_S). On CPUs affected by Meltdown, the kernel must be unmapped while user code runs, which is architecture-specific code ([0008](0008-architecture-neutral-design.md)).
- **IPC cost largely decides performance.** Everything outside the kernel works through system calls and messages between programs (IPC). Day 2 measured two costs:
  - a hardware user/kernel round trip: 109 cycles (about 36 ns);
  - a page-table switch: about 158 cycles with PCID.

  A message round trip between two programs needs two kernel entries and two page-table switches. From those figures, it costs at least about 530 cycles (about 180 ns).
- **A failed driver or service is a failed program.** The kernel reclaims its resources, and the system can restart it.
- **Drivers in user mode need three things from the kernel:**
  - interrupts delivered to them;
  - device memory mapped for them;
  - DMA confined by an IOMMU, without which a device could write anywhere in memory ([06-system](../design/06-system.md)).
- **[0005](0005-unsafe-code-policy.md)'s reason no longer covers user mode.** 0005 assumed that any unsafe code could compromise the whole machine. That still holds for the kernel, but unsafe code in a user-mode program can only harm that program. Whether 0005 stays as written, is narrowed, or is superseded is open ([02-verification](../design/02-verification.md)).
- **Non-Rust software can run natively in user mode** ([0006](0006-non-rust-software.md)). A JIT compiler becomes an ordinary program, so its code generator doesn't need to be trusted. The route for non-Rust software is still open.
- **Part of [0007](0007-abi-calls-and-interfaces.md) (Proposed) no longer applies.** 0007 was written for kernel mode. Its first point, kernel calls as ordinary function calls, is replaced by system calls and IPC. Its other points still apply: what may cross a boundary, one IDL for every interface, and add-only versioning. FIDL, which 0007 uses as its model, was designed for a microkernel ([05-interfaces](../design/05-interfaces.md)).
- **The hardware's user address range bounds a program's address space.** Under [0012](0012-no-memory-cap.md), that is a hardware limit, not a design limit.
- **Self-hosting becomes possible in principle** ([0003](0003-rust-std-no-libc.md)). rustc and LLVM can run as user-mode programs once a route for C++ software exists.
- **The design gains a term for user mode.** It says **user mode** for the privilege level programs run at: ring 3 on x86_64, EL0 on AArch64, U-mode on RISC-V. **Kernel mode** keeps its meaning from [0008](0008-architecture-neutral-design.md).
