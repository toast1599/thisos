# 0208. A new boot loader is installed beside the previous one, which the firmware falls back to

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The boot loader sits outside the system images it chooses between, so their rollback doesn't cover it ([0158](0158-image-try-count.md)). It changes rarely: to replace a working key ([0205](0205-root-and-working-keys.md)), to fix bugs, or to add features. The firmware tries the entries in its boot order in turn, until one loads. Choosing between images needs no firmware runtime services ([0158](0158-image-try-count.md)).

## Options

1. **Beside the old one.** The boot partition keeps the current and the previous boot loader, and the firmware's boot order lists both.
2. **A trial boot through the firmware,** with its one-time boot entry. That needs firmware runtime services, which the kernel would have to call.
3. **A fixed first stage** that chooses between two copies of the boot loader.

## Decision

Option 1.

- The boot partition holds the current boot loader and the previous one.
- The firmware's boot order lists both, the current one first. The boot loader sets the entries up itself, while the firmware's boot services still run.
- The update service installs a new boot loader by keeping the current one as the previous, then writing the new one in its place, once it has checked the new one's signature ([0205](0205-root-and-working-keys.md)).
- If the new boot loader fails to load, or fails Secure Boot's check, the firmware starts the previous one. If it loads but can't boot, the previous one is one choice away in the firmware's boot menu.

## Consequences

- **No firmware runtime services are needed,** as for choosing between images ([0158](0158-image-try-count.md)).
- **A boot loader that loads but misbehaves needs someone at the console.** That's one reason boot loader updates stay rare.
- **An offline attacker can choose the previous boot loader too** ([0209](0209-no-rollback-protection.md)).
- **0158's question on updating the boot loader is answered.**
