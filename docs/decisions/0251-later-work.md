# 0251. Later work stays unscheduled, and the first release states the gaps it leaves

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Several records leave work for later: non-Rust software ([0006](0006-non-rust-software.md), [0186](0186-non-rust-route.md)), randomization ([0061](0061-no-randomization-yet.md)), swap ([0077](0077-swap-in-default-pager.md), [0185](0185-swap-partition.md)), pointer tagging ([0079](0079-pointer-tagging-off.md)), formal proofs ([0018](0018-kernel-assurance.md)), offering our changes upstream ([0150](0150-bootloader-aarch64.md), [0187](0187-pinned-stable-toolchain.md)), encrypting stored data ([0202](0202-offline-attackers.md)), a signature from Microsoft's UEFI certificate authority ([0205](0205-root-and-working-keys.md)) and rollback protection ([0209](0209-no-rollback-protection.md)). Others name smaller later work: 16 and 64 KiB granules ([0069](0069-all-page-sizes.md)), discardable memory ([0076](0076-memory-notice-deadline.md)), finer kernel locking ([0095](0095-one-kernel-lock.md)), IPC channels and larger messages ([0118](0118-calls-and-notifications.md), [0119](0119-message-eight-words.md)), network time ([0173](0173-time-service-entry.md)), file mappings that write back ([0184](0184-shared-file-pages.md)), the IDL's back ends for other languages ([0186](0186-non-rust-route.md)), self-hosting ([0003](0003-rust-std-no-libc.md)) and stronger checks of the toolchain ([0037](0037-kernel-toolchain.md)). Real hardware beyond the laptop, and the laptop's network, are later work too ([0223](0223-qemu-and-laptop-checkpoints.md)).

## Options

1. **Listed, and unscheduled.**
2. **Security items before the first release:** encrypted storage and rollback protection become steps before step 24.
3. **All scheduled,** as steps after step 24.

## Decision

Option 1.

- [08-roadmap](../design/08-roadmap.md) lists each later item with its record, what it needs first, and the earliest step it can follow.
- Items stay unscheduled. A checkpoint may schedule one, through an ordinary record ([0201](0201-changes-after-freeze.md)).
- The first release states the gaps that later work closes, such as stored data kept in the clear ([0202](0202-offline-attackers.md)) and older signed images that still boot ([0209](0209-no-rollback-protection.md)).

## Consequences

- **The roadmap stays the size of the design** ([0222](0222-roadmap-to-first-release.md)).
- **Users of the first release know what it doesn't protect.**
- **Each item's prerequisites are known,** so a checkpoint can schedule it once they're met.
- **Day 8's question on later work is answered,** and so are 0187's, 0202's, 0205's and 0209's on where their later work goes.
