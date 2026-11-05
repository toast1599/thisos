# 0005. Banned patterns

- **Status:** Accepted
- **Date:** 2026-11-05

## Context

The rules for unsafe code in CONTRIBUTING.md say how unsafe code is written, not which constructs to avoid altogether. Milestone 1 and the first milestone 2 work item showed what that leaves open. The framebuffer was a `&'static mut [u8]` from the loader, although the loader's mapping of all physical memory maps the same frames a second time, so the unique reference had an alias, and the compiler was free to treat device memory as ordinary RAM. The PAT setup held a `&'static mut PageTable`, inferred rather than written, over memory the CPU's page walker writes. The trap code first kept its tables, handler and stacks in seven `static mut`s, with the same unsafe write-and-borrow repeated for each. Each was reviewed and passed the lints, which check how unsafe code is documented, not whether it should exist.

## Decision

CONTRIBUTING.md's Banned patterns section lists constructs that no crate uses, each with what to use instead and how it is caught. They cover aliasing (`static mut`, `&'static mut`, references to memory the hardware or another mapping writes, unbounded lifetimes, leaking to get `'static`), values (`transmute`, uninitialized memory, integers turned into pointers, unchecked operations) and escape hatches (locks shared with handlers, `Send` and `Sync` impls that don't encode their invariant, `#[allow]`, unsafe attributes outside the arch crate).

Lints catch what they can: `clippy::allow_attributes`, `clippy::mem_forget` and a `disallowed-methods` list in `clippy.toml`. `cargo xtask ci` runs a source check for `static mut` and `&'static mut`, which no lint catches. Review catches the rest.

The loader is the one exception, for the two references `bootloader_api` hands over. `entry_point!` fixes the entry signature as `fn(&'static mut BootInfo) -> !`, so that reference goes no further than the function `entry!` generates and `boot_info`, which copies out what the kernel needs, and it never leaves the arch crate. `FrameBuffer` keeps the framebuffer's address private, and every accessor returns a slice, so `framebuffer()` makes one `&mut [u8]` with `buffer_mut`, takes its pointer range, and never reads or writes through it. Replacing `entry_point!` with ThisOS's own `_start`, and reading the framebuffer's address from the loader's structure some other way, would remove both, at the cost of checking every `bootloader` upgrade against what that crate does.

Integers become pointers only in the arch crate, for addresses that reach it as integers from the hardware or the loader. No pointer with provenance exists for the kernel stack the frame-pointer walk reads, or for the page tables CR3 names, so a narrower rule would forbid code that has no alternative.

## Consequences

- The framebuffer is an arch type over a raw pointer, and the console draws through the HAL's `Pixels` trait, which its host tests implement over a `Vec`. The PAT setup walks the page tables itself, one atomic access per entry, instead of through `OffsetPageTable`.
- Each kernel-lifetime value written after boot needs a type with interior mutability, and the unsafe code inside it is reviewed once.
- The source check is a text match: it misses an inferred `&'static mut`, and review has to catch that.
- Adding a pattern, or making an exception to one, updates CONTRIBUTING.md and this record.
