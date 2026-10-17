# 05. System calls, IPC and the stable ABI

- **Day:** 5 (Sat 2026-10-17)
- **Status:** In progress

## Covers

The binary interface between programs and the kernel and between programs, how interfaces are defined and versioned, and the security model at the interface level.

## Decided

- [0007](../decisions/0007-abi-calls-and-interfaces.md) ABI: C calling convention, interfaces defined in an IDL, add-only versioning (superseded by [0144](../decisions/0144-c-calls-and-what-crosses.md), never accepted)
- [0015](../decisions/0015-clean-apis.md) APIs and ABIs are designed for this system: inspired by others, never bound to them
- [0020](../decisions/0020-capabilities.md) Authority comes from capabilities
- [0021](../decisions/0021-ipc-simple-upgradable.md) IPC starts simple and modular, and is designed to be upgraded
- [0028](../decisions/0028-namespace-at-spawn.md) Programs find services through a namespace handed over at spawn
- [0029](../decisions/0029-users-as-capabilities.md) A user is the set of capabilities a login service grants
- [0030](../decisions/0030-filesystem-decides-access.md) The filesystem service decides who may open stored data
- [0036](../decisions/0036-ipc-message-max.md) One IPC message has a small fixed maximum size; larger data goes through shared memory
- [0050](../decisions/0050-clients-pay.md) Clients pay for the state services keep for them
- [0058](../decisions/0058-shared-and-handed-over.md) Programs share bulk data through regions that are shared or handed over
- [0059](../decisions/0059-caller-allocates.md) Memory that crosses an interface is allocated by the caller, and its use moves only by handover
- [0064](../decisions/0064-memory-belongs-to-payer.md) Memory belongs to the program it was granted to; lending, sharing and handover move only access (amended by [0084](../decisions/0084-shares-belong-to-accounts.md))
- [0065](../decisions/0065-lost-memory-is-an-error.md) Touching memory that was taken back is an error, not a crash
- [0072](../decisions/0072-registers-only.md) System calls and IPC carry data only in registers; the kernel never reads program data
- [0085](../decisions/0085-clients-pay-cpu-time.md) Clients pay for the CPU time services spend on them (amended by [0121](../decisions/0121-callers-help-busy-services.md))

The kernel boundary and IPC:

- [0117](../decisions/0117-system-library.md) Programs reach the kernel only through the system library (amended by [0160](../decisions/0160-root-task-starts-four.md))
- [0118](../decisions/0118-calls-and-notifications.md) IPC is synchronous calls through endpoints, with notifications for events
- [0119](../decisions/0119-message-eight-words.md) One IPC message carries up to 8 words
- [0120](../decisions/0120-reply-objects.md) Services reply through reply objects
- [0121](../decisions/0121-callers-help-busy-services.md) A caller that finds its service busy lends it its priority and time
- [0122](../decisions/0122-bound-notifications.md) A notification can be bound to a thread
- [0123](../decisions/0123-capabilities-in-messages.md) A message carries up to four capabilities, each copied or moved
- [0124](../decisions/0124-large-calls-split.md) Kernel calls whose data doesn't fit in registers are split, and the system library joins them
- [0125](../decisions/0125-sysret-guard.md) The kernel never returns with `sysret` to an address outside the program half

Capabilities:

- [0126](../decisions/0126-capability-space-radix-table.md) A capability space is a radix table, and capabilities are named by index
- [0127](../decisions/0127-capability-space-per-program.md) Each program has exactly one capability space
- [0128](../decisions/0128-rights-and-badges.md) Capabilities carry a few rights, and endpoint and notification capabilities carry a badge
- [0129](../decisions/0129-revoke-any-capability.md) Revoking any capability destroys every copy made from it
- [0130](../decisions/0130-handlers-changeable.md) A thread's pager and exception handler can be changed through its control right
- [0131](../decisions/0131-namespace-path-prefixes.md) A namespace maps path prefixes to capabilities

Events, faults and scheduling:

- [0132](../decisions/0132-events-notify-then-call.md) An event is a notification, followed by a call for its details
- [0133](../decisions/0133-recovery-point-in-register.md) Accessors keep their recovery point in a register
- [0134](../decisions/0134-scheduler-interface.md) Threads run once the scheduler takes them on, and the kernel reports to it in registers (amended by [0161](../decisions/0161-root-task-before-scheduler.md))
- [0135](../decisions/0135-core-sharing-from-scheduler.md) The scheduler issues core-sharing capabilities
- [0136](../decisions/0136-predictor-flush-flag.md) A program asks for predictor flushes with a flag on its address space

Memory, accounts and identity:

- [0137](../decisions/0137-memory-capability-per-program.md) Each program has its own capability to the memory service, which names it as payer (amended by [0164](../decisions/0164-root-task-memory-pool.md))
- [0138](../decisions/0138-pager-keeps-reservations.md) Pagers keep a program's address-space reservations
- [0139](../decisions/0139-results-fill-and-report.md) A result of unknown size fills what the caller provides, and reports what remains
- [0140](../decisions/0140-accounts-capability-pair.md) An account is a pair of capabilities, one from the scheduler and one from the memory service
- [0141](../decisions/0141-identity-capability.md) The login service gives each session an identity capability

Interface definitions:

- [0142](../decisions/0142-own-idl.md) Interfaces are defined in our own IDL, modeled on FIDL
- [0143](../decisions/0143-add-only-versioning.md) Interfaces only grow, and a tool checks every change
- [0144](../decisions/0144-c-calls-and-what-crosses.md) The system library uses the C calling convention, and only data, capabilities and regions cross between programs
- [0145](../decisions/0145-std-over-native-api.md) std is a layer over the native API
- [0146](../decisions/0146-startup-data-block.md) Startup data is a versioned block defined in the IDL
- [0147](../decisions/0147-elf-declarations-only.md) A program's ELF sections carry only its declarations
- [0148](../decisions/0148-freeze-covers-rules.md) The freeze covers the interface design, and each interface's version 1 is fixed when it's released

## Open questions

None are left in this document. Questions that depend on other topics are listed with those topics:

- **Packages** ([02-verification](02-verification.md)): the package layout, which carries the signature and package metadata [0147](../decisions/0147-elf-declarations-only.md).
- **The compatibility checker in the build** ([07-engineering](07-engineering.md)) [0143](../decisions/0143-add-only-versioning.md).
- **Roadmap** ([08-roadmap](08-roadmap.md)): when the system library, the IDL compiler and the compatibility checker are built, and when each interface's version 1 is released ([0117](../decisions/0117-system-library.md), [0142](../decisions/0142-own-idl.md), [0148](../decisions/0148-freeze-covers-rules.md)).

Questions that depended on the system design were settled with it ([06-system](06-system.md)): how the system library reaches the root task and the first programs ([0153](../decisions/0153-build-unpacks-ramdisk.md), [0154](../decisions/0154-library-entry-table.md), [0160](../decisions/0160-root-task-starts-four.md)), how the scheduling-control capability reaches the scheduler [0161](../decisions/0161-root-task-before-scheduler.md), how the rights to make accounts reach the login service [0163](../decisions/0163-services-hand-first-capabilities.md), interrupts as notifications [0167](../decisions/0167-interrupts-as-notifications.md), the std mapping ([0176](../decisions/0176-paths-and-current-directory.md) to [0182](../decisions/0182-all-of-std.md)), and stored access information [0183](../decisions/0183-access-on-directories.md).
