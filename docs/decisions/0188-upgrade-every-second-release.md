# 0188. The toolchain is upgraded every second stable release

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The toolchain is a pinned stable release ([0187](0187-pinned-stable-toolchain.md)). Each upgrade is deliberate, and reviewed like a kernel change ([0037](0037-kernel-toolchain.md)). Each one also includes an audit of the changes to unsafe code in `core`, `alloc` and std, which stay in the audited set ([0005](0005-unsafe-code-policy.md), [0017](0017-unsafe-code-kernel-and-services.md)), and our patch to std's sources has to be moved onto the new release ([0187](0187-pinned-stable-toolchain.md)). Rust publishes a stable release every six weeks.

## Options

1. **Every second release,** about every 12 weeks.
2. **Every release,** every 6 weeks.
3. **Only when needed,** for a fix or a feature.

## Decision

Option 1.

- The toolchain moves to a new stable release every second release, about every 12 weeks.
- Point releases of the pinned version are taken when they come out.
- A release that fixes a bug affecting thisOS, such as a soundness hole or a miscompilation, can be taken early.
- Each upgrade audits the changes to unsafe code in `core`, `alloc` and std since the last one, moves std's patch onto the new release, and passes the full tests on every supported architecture. Our fork of the `bootloader` crate is rebuilt with it ([0149](0149-bootloader-crate-uefi.md)).

## Consequences

- **Each audit covers about 12 weeks of changes.** There are half as many audits as with every release, each about twice the size.
- **The toolchain trails the newest stable release by at most about 12 weeks,** plus the time an upgrade takes.
- **Point releases and urgent fixes don't wait.** Their reviews cover only their own changes.
- **07's question on how often to upgrade is answered.**
