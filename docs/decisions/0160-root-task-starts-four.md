# 0160. The root task starts four programs itself, and the rest from a boot manifest

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Which programs the root task starts, and in what order, was left to the boot path ([0024](0024-root-task.md)). A thread runs only once the scheduler takes it on ([0134](0134-scheduler-interface.md)), and a program gets its memory capability from the memory service ([0137](0137-memory-capability-per-program.md)). The loader writes each program's startup data and maps the system library into it ([0107](0107-loader-writes-startup-data.md), [0117](0117-system-library.md)), but the loader is itself a program that someone has to start.

## Options

1. **Four programs itself, then a manifest,** through the loader.
2. **The root task starts every program** in the ramdisk itself, with an ELF parser of its own.
3. **Four programs, then a service manager,** which holds most of the authority and starts the rest.

## Decision

Option 1.

- The root task starts the scheduler, the memory service, the default pager and the loader itself, in that order.
- The build unpacks their segments into the ramdisk, as it does the root task's ([0153](0153-build-unpacks-ramdisk.md)), so the root task parses no ELF either. It maps their read-only segments straight from the ramdisk, and copies their writable ones into memory of their own.
- The root task writes these four programs' startup data, and maps the system library into them itself. This amends 0107 and 0117, under which the loader does both for every program.
- The root task then asks the loader to start everything else, from a boot manifest in the ramdisk. The manifest lists each program with its capabilities, namespace, priorities and supervisor. Programs outside the ramdisk start once the package store runs ([0155](0155-ramdisk-reaches-storage.md)).
- The root task supervises the programs it starts ([0104](0104-supervisor.md)).

## Consequences

- **Each step depends only on earlier ones.** The scheduler comes first, so that threads can run ([0161](0161-root-task-before-scheduler.md)). The memory service comes before any program needs a memory capability ([0137](0137-memory-capability-per-program.md)). The default pager and the loader come before the programs they page and build.
- **The build places these four and carries their declarations.** It chooses their addresses, as it does the root task's ([0153](0153-build-unpacks-ramdisk.md)). It also copies what their ELF sections declare ([0147](0147-elf-declarations-only.md)) into the boot manifest, so that the root task can act on it, for example by setting the predictor-flush flag ([0136](0136-predictor-flush-flag.md)).
- **Boot policy is data.** The manifest is defined in the IDL ([0142](0142-own-idl.md)), covered by the image's signature ([0156](0156-boot-loader-checks-image.md)), and changed without changing the root task.
- **The root task builds four address spaces** with the page-table library that the loader and pagers share ([0052](0052-user-mode-page-tables.md)).
- **The root task restarts the services it supervises** by its own policy ([0105](0105-supervisor-reclaims.md)).
- **0024's question on the start sequence is answered.**
