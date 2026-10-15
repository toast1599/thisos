# 0013. Isolation domains: one shared system domain, one checked address space per program

- **Status:** Superseded by [0014](0014-microkernel.md) (never accepted)
- **Date:** 2026-10-15

## Context

[0002](0002-kernel-mode-verified-execution.md) decided that every program runs in kernel mode and is isolated by verification. It did not decide how verification isolates. On 2026-10-15 four mechanisms were compared and measured ([02-verification](../design/02-verification.md)):

- **A.** Trust the compiler. Programs are built by a trusted build service and signed.
- **B.** Check the compiled output with an independent checker. For Rust, only part of this can be built today:
  - **B1** re-checks rustc's MIR at the build service;
  - **B2** ships a checkable form to the device;
  - **B3** checks typed machine code, which is still research.
- **C.** Check the bounds of every memory access in software.
- **D.** Give each program its own page tables, with the kernel and other programs unmapped. Before the program runs, check its machine code: no privileged instructions, and no jump into the middle of an instruction.

These findings drive the choice:

- **Compiler acceptance doesn't stop a hostile author.** 77 open rustc soundness holes apply to this OS. About 50 of them can be reached from safe code, and 31 were opened in 2026.
- **One shared address space doesn't stop speculative reads (Spectre)**, whatever checks the code. Hardening in software against Spectre variant 1 alone cost +44%.
- **C costs +41% to +62%** without a memory cap, and [0012](0012-no-memory-cap.md) forbids a cap.
- **Kernel mode only pays off inside a shared address space.** There, a kernel call is a function call of about 1 ns. With separate address spaces, D's cheapest kernel call (343 cycles) costs about three times a hardware user/kernel round trip (109 cycles).
- **The threat model is broad.** It includes third-party applications, web content and multiple users ([0001](0001-general-purpose-os.md)).

## Options

1. One shared address space for everything, isolated by Rust (A, later B).
2. D for everything: every program and every system component gets its own checked address space.
3. Two kinds of domain, all in kernel mode. The system's own components share one address space, isolated by Rust. Every other program runs in its own address space, checked as in D.
4. The same as option 3, but other programs run in user mode instead of being checked. This is the hybrid with hardware privilege levels that 0002 rejected. It would mean superseding 0002.

## Proposed decision

Option 3.

1. **System domain.**
   - The kernel and the components named as part of the system share one kernel-mode address space. Which components belong there is still open ([01-objective](../design/01-objective.md)).
   - The components are isolated from each other by Rust's safety rules and the closed unsafe set ([0005](0005-unsafe-code-policy.md)).
   - The trusted build service builds them (A). An independent check of rustc's output (B1) is added there later, and adding it changes nothing on the device.
2. **Program domains.** Every other program runs in kernel mode in its own address space, including third-party applications. The kernel and other programs aren't mapped there, except for the kernel entry code.
   - Before any code in a program domain becomes executable, a checker proves two things. The code contains only instructions from an allowed list, so it has no privileged instructions. And no jump can land anywhere except the start of a checked instruction.
   - No memory is ever writable and executable at the same time.
   - Code generated while the program runs is checked the same way before it runs.
3. **Trust.** The compiler isn't trusted for program domains. A bug in rustc, LLVM or a program's copy of std can only affect that program.
4. **Assumptions.** This proposal assumes two things:
   - Speculative reads between programs are in the threat model ([01-objective](../design/01-objective.md)). If they're ruled out, option 1 fits the other decisions better.
   - Page tables count as part of "isolated by verification" ([0002](0002-kernel-mode-verified-execution.md)). Page tables keep the program domains apart, and the checker guarantees that programs can't change them.

## Consequences

- **Two isolation mechanisms** have to be built and audited: Rust-based isolation inside the system domain, and the checker for program domains.
- **The checker is written once per architecture** ([0008](0008-architecture-neutral-design.md)) and joins the trusted base.
  - **AArch64:** small, because every instruction is 4 bytes.
  - **x86-64:** harder, because instruction lengths vary. Code must also follow jump rules that cost 3–4.5% according to published LFI figures, and upstream compilers don't support those rules yet.
- **Calls from a program into the system cost two page-table switches.** Measured: at least 343 cycles (about 114 ns), against about 1 ns for a call inside the system domain. 0002's speed side effect still holds inside the system, but not for programs.
- **Hardware branch-predictor defences don't help.** This CPU's defences (Enhanced IBRS, BHI_DIS_S) separate user mode from kernel mode, so they don't separate program domains from the system domain. The system domain needs retpolines on its indirect branches, or a predictor flush on every entry (545 cycles measured).
- **x86-64 interrupts need extra stacks.** An interrupt that arrives while a program runs doesn't switch stacks, so every interrupt needs its own stack, mapped for each CPU into the running program's address space. AArch64 doesn't need this.
- **The boundary into the system is untrusted.** The kernel keeps every handle in its own tables and copies buffers in before using them. [0007](0007-abi-calls-and-interfaces.md) (Proposed) would apply as written inside the system domain; programs need a checked form of it.
- **[0005](0005-unsafe-code-policy.md) keeps the system domain safe.** In a program domain, unsafe code can only harm its own program, so there 0005 acts as policy.
- **A WebAssembly runtime or other JIT engine can be an ordinary program** ([0006](0006-non-rust-software.md)). Its generated code is checked like any other code, so its code generator doesn't have to be trusted.
- **Other languages are possible.** A program domain can hold code in any language whose compiler follows the checker's rules.
- **Program domains share memory by whole pages**, not by passing objects.
