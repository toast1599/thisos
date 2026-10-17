# 0153. The build unpacks the root task and the system library into the ramdisk

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

The kernel builds the root task's first address space itself ([0052](0052-user-mode-page-tables.md)), including the system library ([0117](0117-system-library.md)), without parsing an executable ([0031](0031-user-mode-loader.md)). The preferred answer was for the boot stage to unpack their ELF segments and hand the kernel a list of them. The `bootloader` crate loads only the kernel and one raw ramdisk ([0149](0149-bootloader-crate-uefi.md)).

## Options

1. **The build unpacks them into the ramdisk,** with a list of their segments.
2. **A fork of the crate unpacks them at boot,** and passes the list in an extended `BootInfo`.
3. **The kernel parses them,** for the root task only, amending 0031.

## Decision

Option 1.

- At build time, the image builder unpacks the root task's and the system library's ELF segments into the ramdisk, page-aligned.
- A fixed-layout header at the start of the ramdisk lists each segment's offset, its address in the root task and its rights. It also gives the root task's entry point and where the library's code is.
- Segments that start out zero-filled, such as the root task's stack and the place for its startup data ([0159](0159-root-task-boot-information.md)), are listed by size only. The kernel backs them with memory of its own choosing.
- The kernel maps only what the list names, and records the library's code range in the root task's address space ([0117](0117-system-library.md)).

## Consequences

- **Nothing parses ELF at boot.** The kernel reads one fixed-layout header. It checks that every segment lies inside the ramdisk, and that none is writable and executable at once ([0054](0054-kernel-enforces-execute.md)).
- **The crate is used unchanged for this.**
- **The build places the root task,** as the loader places every other program ([0061](0061-no-randomization-yet.md)). Like 0052's exception, this covers only what runs before the loader exists. The build leaves the lowest 64 KiB unmapped ([0074](0074-low-64k-unmapped.md)). Randomizing the root task later means the build lists its relocations, and they're applied at boot.
- **The kernel unmaps the ramdisk from its own half** once the root task is set up, so no program data stays in its mappings ([0055](0055-kernel-maps-only-objects.md)).
- **The header is private to the kernel and the image builder.** Both ship in the system image ([0044](0044-system-image-is-os-shipped.md)), so the header can change with any image.
- **The library's pages in the ramdisk serve every program.** The root task receives them as read-and-execute pages ([0159](0159-root-task-boot-information.md)), and they're mapped unchanged into every program ([0154](0154-library-entry-table.md)).
- **The image builder belongs to the build system** ([07-engineering](../design/07-engineering.md)).
- **0052's and 0117's questions on how the kernel maps the root task are answered.**
