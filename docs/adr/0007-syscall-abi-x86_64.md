# 0007. Syscalls on x86_64: SYSCALL, a two-register result, no SIMD state

- **Status:** Accepted
- **Date:** 2026-11-11

## Context

Milestone 3 gives user code its first way into the kernel. design.md left the calling convention open until then. Milestone 3's planning chose SYSCALL and SYSRET over `int 0x80`. What remained was which registers carry what, how an error comes back, and what user state the kernel keeps across a syscall. The kernel is built soft-float and saves no x87, SSE or AVX registers when it switches threads. UEFI leaves SSE on, so two user threads using it would corrupt each other's registers without any trap.

## Decision

- **Entry.** User code enters with SYSCALL. A syscall returns with SYSRET. Interrupts and exceptions from user code return with IRETQ.
- **Number and arguments.** RAX holds the syscall's number, and RDI, RSI, RDX, R10, R8 and R9 hold up to six arguments, in that order. R10 stands in for RCX, which SYSCALL overwrites.
- **Result.** RAX holds 0 for success or an error code, and RDX holds the value. A value can use all 64 bits, and a failed syscall's value is 0.
- **Kept registers.** Every general register but RAX, RDX, RCX and R11 has the same value after the syscall. RCX and R11 hold what SYSRET leaves in them. RFLAGS comes back as user code had it.
- **Numbers and codes** are defined in `thisos-abi`. The kernel answers a number it doesn't know with `Error::UnknownSyscall`.
- **User space** ends one page below the canonical boundary, at `0x7fff_ffff_f000`. No user instruction ends at the boundary, so SYSCALL never saves a non-canonical RIP, which SYSRET would fault on in ring 0 with the user's stack loaded.
- **No SIMD state.** The kernel turns x87, SSE and AVX off: CR0.EM set, CR4.OSFXSR and OSXSAVE clear. Any such instruction traps, and the kernel ends the thread. User crates build soft-float, for `x86_64-unknown-none`, as the kernel does.

## Consequences

- `thisos-rt` turns a result into `Result<u64, Error>` with one compare.
- A syscall that needs more than six arguments or more than one value passes them through memory, which needs user copies.
- A user thread can't use floating point in hardware, or any library built for a target with SSE, until the kernel saves SIMD state per thread. That is due before milestone 7's `std`, and it gets a new record.
- The page below `0x8000_0000_0000` can never be mapped.
- aarch64's convention is decided when user mode is ported there ([0006](0006-aarch64-behind-x86_64.md)).
