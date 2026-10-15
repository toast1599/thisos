# 06. Boot, user-mode drivers, std mapping, non-Rust software

- **Day:** 6 (Mon 2026-10-19)
- **Status:** Not started

## Covers

How the system starts, how it talks to hardware, and how the Rust standard library maps onto the OS.

## Decided

- [0004](../decisions/0004-not-unix.md) Not Unix-like
- [0006](../decisions/0006-non-rust-software.md) Non-Rust software is supported later; route still open
- [0015](../decisions/0015-clean-apis.md) APIs and ABIs are designed for this system: inspired by others, never bound to them
- [0024](../decisions/0024-root-task.md) At boot, one root task receives every initial capability
- [0032](../decisions/0032-minimal-hardware-discovery.md) The kernel reads only the hardware information it needs to start

## Open questions

- **Boot path:** UEFI, Limine, Multiboot2, or something else. Under [0024](../decisions/0024-root-task.md), the kernel starts one root task with every initial capability. Which programs the root task starts, and in what order, belongs here too.
- **Driver model:**
  - under [0014](../decisions/0014-microkernel.md), drivers run in user mode. Open: how interrupts reach them, and how device memory is mapped for them. Under [0032](../decisions/0032-minimal-hardware-discovery.md), they learn about devices from a user-mode service that parses ACPI tables or device trees.
  - protecting against DMA from devices (IOMMU)
- **std mapping:**
  - the target name and its `target_family` value
  - which std features are supported
  - how paths, environment variables, arguments and sockets map onto the native model. Under [0015](../decisions/0015-clean-apis.md), std's platform layer does the translating; the native model isn't shaped to fit std's Unix-derived parts. Under [0020](../decisions/0020-capabilities.md), paths have to resolve through capabilities the program holds.
- **Stored access information.** Under [0030](../decisions/0030-filesystem-decides-access.md), the filesystem service decides who may open stored data. What access information does stored data carry for that policy?
- **Route for non-Rust software.** This is later work, but the design should not block it ([0006](../decisions/0006-non-rust-software.md)). Under [0014](../decisions/0014-microkernel.md), non-Rust software can run natively in user mode. The open part is how its runtime reaches the OS. Under [0015](../decisions/0015-clean-apis.md), any compatibility it needs comes from a separate user-mode layer, not from the native API.
