# 0191. A list of audited crates, enforced by a compiler wrapper, keeps unsafe code in the audited set

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The kernel and the OS's own services keep their unsafe code in one closed, audited set ([0017](0017-unsafe-code-kernel-and-services.md)). 0005's build rules carry over: every crate outside the set gets `-F unsafe_code` and `-Zallow-features=` ([0005](0005-unsafe-code-policy.md)). The set holds the kernel's unsafe code, the boot loader's ([0149](0149-bootloader-crate-uefi.md)), the system library's wrappers ([0117](0117-system-library.md)), the accessors ([0133](0133-recovery-point-in-register.md)), the bindings' unsafe code ([0142](0142-own-idl.md)), the `thisos` crate's ([0178](0178-native-api-crate.md)), and `core`, `alloc` and std. 02 asked for a list of these parts, each with an audit record ([02-verification](../design/02-verification.md)).

Cargo caps the lints of dependencies from registries, vendored ones included, at `allow`. A plain `-F unsafe_code` would then do nothing for them.

## Options

1. **A list and a compiler wrapper.** A list in the repository names the crates that may contain unsafe code, each with its audit record. A wrapper around the compiler applies the rules to every other crate of the OS's own code, dependencies included.
2. **Attributes and a scan.** Our crates declare `#![forbid(unsafe_code)]`, CI checks the attribute is there, and a scanner checks dependencies.
3. **Manifest lints.** Cargo's lint table in each of our crates, which doesn't reach dependencies.

## Decision

Option 1.

- A list in the repository names every crate allowed to contain unsafe code. Each entry links to the crate's audit record. For a third-party crate, the entry names its version and hash.
- The build runs the compiler through a wrapper. For every crate of the OS's own code that isn't on the list, dependencies included, the wrapper adds `-F unsafe_code` and `-Zallow-features=`, and removes Cargo's lint cap.
- A crate joins the list only with an audit record ([0017](0017-unsafe-code-kernel-and-services.md)).
- The list is 02's list of the audited set. It's filled in as the parts are written.

## Consequences

- **The compiler decides what counts as unsafe.** The check that rejects `unsafe` blocks also rejects `global_asm!`, `#[unsafe(no_mangle)]` and the like ([0005](0005-unsafe-code-policy.md)).
- **Dependencies can't slip past the rule.** A vendored crate with unsafe code fails to build unless it's on the list.
- **Third-party programs aren't affected.** The wrapper applies only to the OS's own code: the system image and the boot loader ([0044](0044-system-image-is-os-shipped.md), [0149](0149-bootloader-crate-uefi.md)).
- **Audit records live in the repository,** beside the list. Their form, and the review rules, are Day 8's ([08-roadmap](../design/08-roadmap.md)).
- **02's question on the audited set is answered** for how the set is kept.
