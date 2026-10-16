# 0123. A message carries up to four capabilities, each copied or moved

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Capabilities are passed explicitly ([0020](0020-capabilities.md)), and IPC is how they travel between running programs. Data travels only in registers ([0072](0072-registers-only.md)), and a message carries up to 8 data words ([0119](0119-message-eight-words.md)). Handover needs the sender to lose its access ([0058](0058-shared-and-handed-over.md)), and revoking follows the kernel's records of what was made from what ([0053](0053-revoke-destroys-derived.md)).

## Options

1. **Up to four per message, each copied or moved.**
2. **One per message, copied only.**

## Decision

Option 1.

- A message carries up to four capabilities. Each is named in one of its data words.
- The sender marks each capability as copied or moved.
- A copy is recorded as made from the sender's capability, so the sender can revoke it later ([0129](0129-revoke-any-capability.md)).
- A move takes the capability out of the sender's capability space, and keeps its place in the records.
- Sending capabilities needs the grant right on the endpoint capability used ([0128](0128-rights-and-badges.md)).
- Received capabilities land in empty slots the receiver named when it waited. Their indexes replace the names in the data words it receives.

## Consequences

- **Handover is a move** ([0058](0058-shared-and-handed-over.md)). The kernel first revokes everything made from the moved capability, including the sender's mappings and copies, so the sender keeps no access through it.
- **Handover protects the receiver from the sender, not from the payer.** The payer can still take the memory back ([0064](0064-memory-belongs-to-payer.md)), which the receiver sees as an error ([0065](0065-lost-memory-is-an-error.md)). A sender that was given two separate copies of a region keeps the other one, so protocols that rely on handover take regions from their payer.
- **Lending raw memory is sending a copy** ([0050](0050-clients-pay.md)), and taking it back is revoking that copy ([0129](0129-revoke-any-capability.md)).
- **Each capability uses a data word,** so a message carrying four capabilities has four words left for data.
- **A receiver controls where capabilities land.** One that names no empty slots can't be sent capabilities.
- **A spawner confines a program through grant rights.** A program can pass capabilities only through endpoints whose capabilities carry the grant right ([0022](0022-explicit-spawn.md)).
