# 0270. The first release states the offline gap in full, and the machine-bound key needs measured boot or the owner's secret

- **Status:** Accepted
- **Date:** 2026-10-19

## Context

Attackers who get the machine while it's off are in scope, and stored data stays in the clear until later work ([0202](0202-offline-attackers.md)). The owner's approvals of signers ([0206](0206-owner-approves-signers.md)) and the grants on directories ([0183](0183-access-on-directories.md)) are stored data. So an attacker with the disk can approve a signer of their own, and put a program they signed under `/bin`, where the user's commands find it ([0179](0179-command-copies-namespace.md)). The user then runs it at the next login. On the laptop, that disk is a USB disk ([0224](0224-laptop-usb-storage.md)). 0202 and 0206 each state a part of this, but no record states the whole, as the audit found from their text ([0256](0256-audit-findings-as-evidence.md)).

0202's later work names "a key bound to the machine, for example one sealed to its TPM". A key sealed with no policy on the measured boot state unseals for any software on that machine, including another system that the attacker boots from USB. Measured boot isn't part of the design ([0156](0156-boot-loader-checks-image.md)).

## Options

1. **The gap stated in full,** and the later work corrected.
2. **The data protected before the first release,** with measured boot and a key sealed to it.
3. **Left as is.**

## Decision

Option 1.

- The first release states the whole gap. An attacker with the disk can make the system run a program of their choosing, as a user, through the signer approvals and the directory grants. This joins 0251's list of gaps.
- The key that later protects data belonging to no user must be sealed to the measured boot state, through a TPM policy on its measurements, or derived from a secret only the owner knows.
- This amends 0202, whose later work gains that requirement, and 0251, whose list of gaps gains this one.

## Consequences

- **Users of the first release know what an attacker with the disk can do,** not only that stored data is in the clear.
- **The later work can't settle for a plain sealed key.** It brings measured boot with it, or the owner's secret.
- **08-roadmap's later work for encrypted stored data needs measured boot,** or the owner's secret, first.
- **Step 24 is affected** ([08-roadmap](../design/08-roadmap.md)).
