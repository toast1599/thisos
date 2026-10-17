# 0209. Older signed images still boot, and rollback protection is later work

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The boot loader refuses any image that isn't correctly signed ([0156](0156-boot-loader-checks-image.md)), and the boot partition keeps the current and the previous system image ([0158](0158-image-try-count.md)). Attackers who get the machine while it's off are in scope ([0202](0202-offline-attackers.md)). Such an attacker, or a compromised update service, could put back an older signed image with known holes. Protection would keep the highest version marked good in a firmware variable that only boot-time code can write, and refuse any image below it.

## Options

1. **A minimum version.** Each image carries a security version, and the boot loader refuses images below the highest one marked good.
2. **No rollback protection.**

For the gap this leaves:

- **Later work,** with images versioned from the start.
- **No plan** to add protection.

## Decision

Option 2, with protection as later work.

- Any correctly signed system image boots, whatever its age. So does any boot loader that Secure Boot accepts.
- Each system image carries a version number from the start, so that protection can be added without changing the image format.
- Rollback protection goes on Day 8's list of later work ([08-roadmap](../design/08-roadmap.md)).

## Consequences

- **The boot loader keeps no state of its own,** and needs no firmware variable.
- **An offline attacker can put back an older signed image or boot loader,** and use its known holes. Verified boot still refuses anything unsigned or altered ([0202](0202-offline-attackers.md)).
- **Revoked keys share the gap.** A replaced boot loader can be put back, along with the statements it still accepts ([0205](0205-root-and-working-keys.md)).
- **Adding protection later changes the boot loader, not the images.**
