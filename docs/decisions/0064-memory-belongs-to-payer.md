# 0064. Memory belongs to the program it was granted to; lending, sharing and handover move only access

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Memory moves between programs: it's lent to services ([0050](0050-clients-pay.md)), shared or handed over ([0058](0058-shared-and-handed-over.md)), and passed across interfaces ([0059](0059-caller-allocates.md)). Fair shares ([0049](0049-memory-fair-share.md)) need to know whose memory it is, and memory has to be revocable by force ([0053](0053-revoke-destroys-derived.md)).

## Options

1. **Always the payer.** Memory counts against the program it was granted to, which can always take it back.
2. **Handover moves payment.** A handover moves the charge, and the right to take the memory back, to the receiver.

## Decision

Option 1.

- Memory always counts against the program it was granted to, wherever its capabilities travel. That program is its payer.
- The payer can always take the memory back.
- Lending, sharing and handover move only access, never the charge or the right to take the memory back.
- When [0059](0059-caller-allocates.md) says each piece of memory has exactly one owner, that owner is the payer.

## Consequences

- **Fair shares count memory by payer** ([0049](0049-memory-fair-share.md)). Lent, shared and handed-over memory counts against the program that pays for it.
- **A handover doesn't end the sender's hold.** After a handover ([0058](0058-shared-and-handed-over.md)), the sender can no longer read or change the region, but as payer it can take the memory back. The receiver sees that as losing the region ([0065](0065-lost-memory-is-an-error.md)).
- **Memory never moves between accounts.** The memory service needs no way to move it, and the kernel's records of what was made from what ([0053](0053-revoke-destroys-derived.md)) stay as they are.
- **Programs that use others' memory must expect to lose it** ([0065](0065-lost-memory-is-an-error.md)).
