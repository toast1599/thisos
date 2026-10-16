# 0117. Programs reach the kernel only through the system library

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Programs enter the kernel with the architecture's system-call instruction, and all data travels in registers ([0072](0072-registers-only.md)). If programs, through std, executed that instruction themselves, the call numbers and the register layout for each architecture would become a stable ABI that could never change. [0021](0021-ipc-simple-upgradable.md) requires that programs don't depend on the IPC mechanism, so that it can be upgraded.

Fuchsia maps a vDSO into every process. Its functions are the only stable system interface, and the kernel rejects system calls made from anywhere else. OpenBSD now pins system calls to its C library too. Go made raw system calls on macOS and OpenBSD until both systems forced it through their libraries.

## Options

1. **A system library, enforced.** The kernel accepts system calls only from it.
2. **Raw system calls.** The call numbers and register layout are the stable ABI, as on Linux.
3. **A system library, not enforced.** Programs may also make system calls themselves.

## Decision

Option 1.

- The loader maps one system library, shipped with the OS, into every program.
- Its functions, called with the C calling convention ([0144](0144-c-calls-and-what-crosses.md)), are the stable ABI between programs and the kernel.
- System-call numbers and the registers each call uses are private between the library and the kernel. They're documented, but they may change with any system image.
- Each address space records where the library's code is when the address space is made, and the record never changes. The kernel refuses a system call made from anywhere else.

## Consequences

- **The IPC mechanism can change under programs** ([0021](0021-ipc-simple-upgradable.md)). Only the library and the kernel change, and they change together.
- **The library and the kernel always match.** Both ship in the system image and update with it ([0042](0042-updates.md), [0044](0044-system-image-is-os-shipped.md)).
- **Every system call costs one more indirect call,** about 1 ns on the development machine ([02-verification](../design/02-verification.md)).
- **The library is OS code with unsafe code in it.** Its system-call wrappers belong to the audited set ([0017](0017-unsafe-code-kernel-and-services.md)).
- **Which registers each call uses on each architecture is documented with the library,** not frozen ([0008](0008-architecture-neutral-design.md)). This answers 05's question on system calls, apart from the `sysret` hazard ([0125](0125-sysret-guard.md)).
- **Programs learn the library's address from their startup data** ([0146](0146-startup-data-block.md)).
- **The kernel maps the library into the root task itself,** with the rest of the root task's first address space ([0052](0052-user-mode-page-tables.md)). How is designed with the boot path ([06-system](../design/06-system.md)).
- **Non-Rust runtimes reach the kernel the same way** ([0006](0006-non-rust-software.md)), and so does code that a JIT engine generates.
