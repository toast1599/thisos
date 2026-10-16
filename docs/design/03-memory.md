# 03. Memory model

- **Day:** 3 (Thu 2026-10-15)
- **Status:** In progress

## Covers

Address spaces, memory layout, allocation, and how programs share data.

## Decided

- [0012](../decisions/0012-no-memory-cap.md) No fixed limit on program memory
- [0014](../decisions/0014-microkernel.md) Microkernel: a minimal, audited kernel in kernel mode; everything else in user mode (amended by [0082](../decisions/0082-corrections.md)). Each program has its own address space.
- [0025](../decisions/0025-user-mode-memory.md) Physical memory is managed from user mode
- [0026](../decisions/0026-programs-pay-kernel-objects.md) Programs pay for their kernel objects with memory they hold
- [0027](../decisions/0027-user-mode-pagers.md) Page faults are handled by user-mode pagers (amended by [0081](../decisions/0081-pager-per-thread.md))
- [0043](../decisions/0043-executable-memory-capability.md) Making memory executable at run time needs a capability
- [0047](../decisions/0047-kernel-mapping.md) The kernel stays mapped in every address space unless the CPU requires otherwise
- [0100](../decisions/0100-cpu-flaw-mitigations.md) The kernel applies each CPU's documented mitigations for speculative-execution flaws

How memory is handed out:

- [0048](../decisions/0048-reserve-on-grant.md) Memory is reserved when it's granted; the system never promises more than it has
- [0049](../decisions/0049-memory-fair-share.md) When memory runs short, it's shared fairly within each memory priority (amended by [0084](../decisions/0084-shares-belong-to-accounts.md))
- [0084](../decisions/0084-shares-belong-to-accounts.md) Fair shares of CPU time and memory belong to accounts (amended by [0092](../decisions/0092-scheduler-keeps-time.md))
- [0050](../decisions/0050-clients-pay.md) Clients pay for the state services keep for them
- [0062](../decisions/0062-memory-priority-capability.md) Memory priority is a capability of its own
- [0064](../decisions/0064-memory-belongs-to-payer.md) Memory belongs to the program it was granted to; lending, sharing and handover move only access (amended by [0084](../decisions/0084-shares-belong-to-accounts.md))
- [0071](../decisions/0071-three-memory-programs.md) The root task, the memory service and the default pager are three separate programs
- [0076](../decisions/0076-memory-notice-deadline.md) Programs over their fair share get a notice, then a deadline
- [0077](../decisions/0077-swap-in-default-pager.md) Swap is a feature of the default pager; it comes later and is encrypted (amended by [0082](../decisions/0082-corrections.md))

Kernel mechanisms:

- [0051](../decisions/0051-kernel-clears-memory.md) The kernel clears memory before it's made into anything new
- [0052](../decisions/0052-user-mode-page-tables.md) Page tables are kernel objects that user mode builds
- [0053](../decisions/0053-revoke-destroys-derived.md) Revoking memory destroys everything made from it
- [0054](../decisions/0054-kernel-enforces-execute.md) The kernel enforces the right to make memory executable
- [0055](../decisions/0055-kernel-maps-only-objects.md) The kernel maps its own memory and kernel objects, not all of physical memory (amended by [0096](../decisions/0096-object-window-lazy-unmap.md))
- [0060](../decisions/0060-kernel-stack-per-core.md) The kernel has one stack per CPU core, and its worst-case depth is checked at build time
- [0068](../decisions/0068-no-kernel-heap.md) The kernel has no heap (amended by [0097](../decisions/0097-user-mode-starts-cores.md))
- [0072](../decisions/0072-registers-only.md) System calls and IPC carry data only in registers; the kernel never reads program data
- [0075](../decisions/0075-user-guards-always-on.md) The kernel's guards against touching or running program memory are always on
- [0096](../decisions/0096-object-window-lazy-unmap.md) The object window keeps memory mapped until it becomes a program's pages

Pagers and sharing:

- [0056](../decisions/0056-pager-chosen-at-spawn.md) A program's pager is chosen when it's spawned (amended by [0081](../decisions/0081-pager-per-thread.md))
- [0057](../decisions/0057-pager-fault-chain.md) A pager's own faults go to its pager, up a chain that ends at the root task
- [0058](../decisions/0058-shared-and-handed-over.md) Programs share bulk data through regions that are shared or handed over
- [0059](../decisions/0059-caller-allocates.md) Memory that crosses an interface is allocated by the caller, and its use moves only by handover
- [0065](../decisions/0065-lost-memory-is-an-error.md) Touching memory that was taken back is an error, not a crash
- [0081](../decisions/0081-pager-per-thread.md) Page faults go to the faulting thread's pager

Layout:

- [0061](../decisions/0061-no-randomization-yet.md) Nothing is randomized at first, and the design keeps randomization possible
- [0063](../decisions/0063-widest-addressing.md) The kernel turns on the widest virtual addressing the CPU offers
- [0066](../decisions/0066-kernel-upper-half.md) The kernel sits in the upper half of every address space, and programs in the lower half
- [0067](../decisions/0067-object-window-offset.md) Kernel objects are mapped at their physical address plus a fixed offset (amended by [0096](../decisions/0096-object-window-lazy-unmap.md))
- [0069](../decisions/0069-all-page-sizes.md) The kernel supports every page size the hardware offers
- [0070](../decisions/0070-position-independent-programs.md) Programs are built position-independent
- [0073](../decisions/0073-kernel-layout-in-implementation.md) The kernel's internal layout is chosen during implementation
- [0074](../decisions/0074-low-64k-unmapped.md) The lowest 64 KiB of every program's half stays unmapped
- [0078](../decisions/0078-low-addresses-on-request.md) Programs that need low addresses ask for them
- [0079](../decisions/0079-pointer-tagging-off.md) Hardware pointer tagging stays off for now

## Open questions

None are left in this document. Memory questions that depend on other topics are listed with those topics:

- **File-backed memory, DMA and swap storage** ([06-system](06-system.md)).

Memory requests and notices were settled with the interfaces ([05-interfaces](05-interfaces.md)): payers [0137](../decisions/0137-memory-capability-per-program.md), reservations and low addresses [0138](../decisions/0138-pager-keeps-reservations.md), notices [0132](../decisions/0132-events-notify-then-call.md), and lost memory as an error [0133](../decisions/0133-recovery-point-in-register.md).
