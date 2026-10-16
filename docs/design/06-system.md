# 06. Boot, user-mode drivers, std mapping, non-Rust software

- **Day:** 6 (Sun 2026-10-18)
- **Status:** Not started

## Covers

How the system starts, how it talks to hardware, and how the Rust standard library maps onto the OS.

## Decided

- [0004](../decisions/0004-not-unix.md) Not Unix-like
- [0006](../decisions/0006-non-rust-software.md) Non-Rust software is supported later; route still open (amended by [0082](../decisions/0082-corrections.md))
- [0015](../decisions/0015-clean-apis.md) APIs and ABIs are designed for this system: inspired by others, never bound to them
- [0024](../decisions/0024-root-task.md) At boot, one root task receives every initial capability
- [0032](../decisions/0032-minimal-hardware-discovery.md) The kernel reads only the hardware information it needs to start
- [0071](../decisions/0071-three-memory-programs.md) The root task, the memory service and the default pager are three separate programs
- [0097](../decisions/0097-user-mode-starts-cores.md) User mode finds the CPUs, and the root task asks the kernel to start each one

## Open questions

- **Boot path:** UEFI, Limine, Multiboot2, or something else. Under [0024](../decisions/0024-root-task.md), the kernel starts one root task with every initial capability, with all of its memory mapped [0057](../decisions/0057-pager-fault-chain.md). The kernel builds that first address space itself [0052](../decisions/0052-user-mode-page-tables.md) without parsing an executable [0031](../decisions/0031-user-mode-loader.md), and how is open. The root task starts the memory service and the default pager and handles their faults [0071](../decisions/0071-three-memory-programs.md). How it backs their memory is open, and so is how the right to grant memory priorities reaches it from the memory service [0062](../decisions/0062-memory-priority-capability.md). Which programs the root task starts, and in what order, belongs here too, including how the scheduler is started, how its threads get a priority above every other [0114](../decisions/0114-scheduler-priority.md), and what runs before it exists [0088](../decisions/0088-user-mode-scheduler.md). The kernel starts on one core, and the root task asks it to start the others [0097](../decisions/0097-user-mode-starts-cores.md): how the root task names a CPU, how a CPU is started on each architecture, and what happens if the memory behind a running core is taken back are open.
- **Driver model:**
  - under [0014](../decisions/0014-microkernel.md), drivers run in user mode. Open: how interrupts reach them, and how device memory is mapped for them. Under [0032](../decisions/0032-minimal-hardware-discovery.md), they learn about devices from a user-mode service that parses ACPI tables or device trees.
  - protecting against DMA from devices (IOMMU)
  - DMA and revoke: how memory a device can reach is mapped for it through the IOMMU, and how revoking [0053](../decisions/0053-revoke-destroys-derived.md) and clearing [0051](../decisions/0051-kernel-clears-memory.md) cover those mappings
- **std mapping:**
  - the target name, which takes its OS name from [0116](../decisions/0116-name-thisos.md), and its `target_family` value
  - which std features are supported
  - how paths, environment variables, arguments and sockets map onto the native model. Under [0015](../decisions/0015-clean-apis.md), std's platform layer does the translating; the native model isn't shaped to fit std's Unix-derived parts. Under [0020](../decisions/0020-capabilities.md), paths have to resolve through capabilities the program holds.
- **Stored access information.** Under [0030](../decisions/0030-filesystem-decides-access.md), the filesystem service decides who may open stored data. What access information does stored data carry for that policy?
- **Route for non-Rust software.** This is later work, but the design should not block it ([0006](../decisions/0006-non-rust-software.md)). Under [0014](../decisions/0014-microkernel.md), non-Rust software can run natively in user mode. The open part is how its runtime reaches the OS. Under [0015](../decisions/0015-clean-apis.md), any compatibility it needs comes from a separate user-mode layer, not from the native API.
- **File-backed memory.** How files and package contents are mapped into programs. That covers which pager supplies their pages [0056](../decisions/0056-pager-chosen-at-spawn.md), whether programs running the same package share its code pages, and who pays for pages that several programs share ([0050](../decisions/0050-clients-pay.md), [0064](../decisions/0064-memory-belongs-to-payer.md)).
- **The time service** ([0112](../decisions/0112-time-service.md)). How programs reach it, and how it reads the hardware clocks.
- **Swap storage** ([0077](../decisions/0077-swap-in-default-pager.md)). Where swapped pages are stored, and how the default pager reaches the storage service.
