# 0058. Programs share bulk data through regions that are shared or handed over

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

IPC messages have a small maximum size ([0036](0036-ipc-message-max.md)), so bulk data goes through shared memory. While the sender can still write to shared memory, data the receiver has checked can change after the check.

## Options

1. **Shared and handed over.** Regions can be shared, or handed over so the sender loses access.
2. **Shared regions only.** Receivers that need to rely on the contents copy them first.

## Decision

Option 1.

- A region is shared by passing a capability for it. Each side's rights are set separately, for example read-only for the receiver.
- A region can also be handed over. The sender then loses all access to its contents, so it can't change the data after the receiver has checked it.
- A sender that pays for the region can still take the memory back ([0064](0064-memory-belongs-to-payer.md)). The receiver then loses the region and gets errors when it touches it ([0065](0065-lost-memory-is-an-error.md)), and the kernel clears the memory before the sender can use it again ([0051](0051-kernel-clears-memory.md)).
- Shared regions suit streams and device buffers. Handover suits one-off transfers.

## Consequences

- **Handover relies on revoke** ([0053](0053-revoke-destroys-derived.md)). The kernel removes all of the sender's mappings and capabilities for the region, which costs TLB flushes on the cores where the sender ran.
- **Receivers of shared regions copy first.** While the sender keeps write access, a receiver copies whatever it has to rely on.
- **Handover moves use, not payment** ([0064](0064-memory-belongs-to-payer.md)). It's how use of memory moves across interfaces ([0059](0059-caller-allocates.md)).
- **03's sharing question is answered.**
