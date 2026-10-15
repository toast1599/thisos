# 0055. The kernel maps its own memory and kernel objects, not all of physical memory

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Many kernels, Linux and seL4 among them, map all of physical memory into the kernel's half of every address space. Speculative leaks between programs are in scope ([0016](0016-threat-model-scope.md)), and the kernel stays mapped ([0047](0047-kernel-mapping.md)). With all memory mapped, every program's data is within reach of a speculative read in the kernel.

## Options

1. **Only kernel objects.** The kernel maps its own code and data and the kernel objects made from program memory.
2. **All physical memory,** as Linux and seL4 do.

## Decision

Option 1.

- The kernel's mappings cover its own code and data, and the kernel objects made from program memory.
- Program data is not mapped in the kernel's half.

## Consequences

- **A speculative read in the kernel can't reach other programs' data** through the kernel's mappings.
- **Kernel objects change kernel mappings.** Making a kernel object maps it, and destroying one unmaps it. Kernel mappings are shared by every address space, so unmapping needs a TLB flush on every core. This happens far less often than IPC.
- **Kernel objects sit at their physical address plus a fixed offset** ([0067](0067-object-window-offset.md)).
- **The kernel never reads program data** ([0072](0072-registers-only.md)). It touches program memory only to clear it ([0051](0051-kernel-clears-memory.md)), through a brief mapping of its own.
