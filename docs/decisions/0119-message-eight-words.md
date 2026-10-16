# 0119. One IPC message carries up to 8 words

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

One IPC message has a small fixed maximum size ([0036](0036-ipc-message-max.md)). It travels in registers ([0072](0072-registers-only.md)), so the maximum is what fits on every architecture. x86_64 is the tightest. Its system-call instruction takes `rcx` and `r11`, and the stack pointer stays the program's, which leaves 13 general registers. The busiest call is a service replying to one client while it waits for the next. It needs about 5 registers for control: the call number, the endpoint, the reply object, the message tag, and where received capabilities go.

## Options

1. **8 words (64 bytes),** which uses every register x86_64 has left on the busiest call.
2. **6 words (48 bytes),** which leaves two registers spare on x86_64 for control fields a later version might need.

## Decision

Option 1.

- One message carries up to 8 data words, 64 bytes.
- The maximum is the same on every architecture.
- A tag travels with the data. It names the method, and says how many data words follow and how many of them name capabilities ([0123](0123-capabilities-in-messages.md)).

## Consequences

- **A message never depends on where it runs.** Interfaces and bindings see the same size on every architecture ([0008](0008-architecture-neutral-design.md)).
- **Capabilities in a message use its data words** ([0123](0123-capabilities-in-messages.md)).
- **Anything larger goes through shared regions** ([0036](0036-ipc-message-max.md), [0058](0058-shared-and-handed-over.md)). The IDL's bindings do the splitting ([0142](0142-own-idl.md)).
- **On x86_64, the busiest call has no register to spare.** A later control field would have to come from vector registers or a new call.
- **The size can be raised later** ([0036](0036-ipc-message-max.md)), for example with vector registers, behind the system library ([0117](0117-system-library.md)).
- **0036's and 0072's question on the exact size is answered.**
