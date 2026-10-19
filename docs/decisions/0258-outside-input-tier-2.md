# 0258. Code that parses outside input, or carries what users type, is reviewed in tier 2

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Tier 2 gets a full review against the threat model, and tier 3 a normal review ([0242](0242-review-tiers.md)). Attackers who reach a running machine from outside are now in scope ([0257](0257-outside-attackers.md)). Yet tier 3 holds the network service, the USB core and its drivers, the console and the keyboard drivers ([0242](0242-review-tiers.md)), as the audit found from 0242's lists ([0256](0256-audit-findings-as-evidence.md)).

The network service holds every program's sockets ([0181](0181-network-service.md)). On the laptop, the system's storage sits on a USB disk ([0224](0224-laptop-usb-storage.md)). The console carries everything users type into programs, and draws every program's output. Login credentials never pass through it ([0260](0260-credentials-path.md)). With offline attackers in scope, the filesystem already has to treat what it reads from disk as hostile ([0202](0202-offline-attackers.md)).

## Options

1. **The network service, the USB stack, the console and the keyboard drivers.**
2. **The same, plus block storage and the storage drivers.**
3. **The keyboard drivers only,** with the console left in tier 3.

## Decision

Option 1.

- Tier 2 also covers the network service, the xHCI driver, the USB core, the USB mass-storage and keyboard drivers, the PS/2 keyboard driver, and the console.
- The network service's own code is reviewed in tier 2. The vendored TCP/IP stack keeps its review through cargo-vet, and imported reviews still count for it, since it isn't in the audited set ([0243](0243-audit-records-and-crate-reviews.md), [0244](0244-imported-crate-reviews.md)).
- The tier 2 review of these parts also checks how they handle input from outside.
- Block storage and the NVMe driver stay in tier 3. The filesystem treats what they return as hostile, and is fuzzed with crafted disk images ([0259](0259-outside-input-fuzzed.md)).
- This amends 0242, whose tier 2 gains these parts.

## Consequences

- **Outside input meets a full review** in the code that parses it first.
- **The console's review covers what users type into programs,** while credentials stay out of it ([0260](0260-credentials-path.md)).
- **A compromised storage driver gains nothing** that an attacker with the disk lacks ([0202](0202-offline-attackers.md)).
- **Review takes longer:** about 20 more hours at step 18, about 12 at step 22, and a few at step 21 ([08-roadmap](../design/08-roadmap.md)).
- **Steps 18, 21 and 22 are affected.**
