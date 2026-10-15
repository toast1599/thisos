# 0017. Unsafe code: one closed, audited set for the kernel and the OS's own services

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

[0005](0005-unsafe-code-policy.md) confined all unsafe code to one closed, audited set, because with every program in kernel mode any unsafe code could compromise the machine. Under the microkernel ([0014](0014-microkernel.md)), unsafe code in a user-mode program can only harm that program. But the OS's own drivers and services hold the data and devices that other programs depend on.

## Options

1. Keep 0005 for every program, third-party ones included.
2. Apply it to the kernel and to the drivers and services the OS ships.
3. Apply it to the kernel only.

## Decision

Option 2.

- **What the set covers.** The closed, audited set contains the kernel's unsafe code, plus shared safe wrappers that the OS's own drivers and services use, for example for device memory. `core`, `alloc` and `std` stay in the set, as in 0005.
- **The OS's own code.** The drivers and services the OS ships contain no other unsafe code.
- **Third-party programs** follow normal Rust practice and may contain unsafe code, which can only harm them.

## Consequences

- **0005 is superseded.** Its evidence, on what `forbid(unsafe_code)` and `-Zallow-features=` catch, still holds. So do its build rules, which now apply to the kernel and to the OS's own drivers and services.
- **Crates for the OS's own code.** Its drivers and services can use only crates without unsafe code, or crates whose unsafe parts have been brought into the audited set. Third-party programs can use the crate ecosystem freely.
- **The wrappers need audits.** The shared wrappers, for device memory, DMA buffers and the like, are part of the audited set and need audit records ([02-verification](../design/02-verification.md)).
- **"Shipped by the OS" needs a definition:** the list of components the OS ships.
