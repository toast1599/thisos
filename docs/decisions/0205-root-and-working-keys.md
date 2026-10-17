# 0205. An offline root key names the working keys, and the boot loader carries only the root

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The boot loader has to know the system image's key ([0156](0156-boot-loader-checks-image.md)). 02 asked how signing keys are managed, and who signs the system image and the boot loader ([02-verification](../design/02-verification.md)). UEFI Secure Boot checks the boot loader before it runs, against keys in the firmware ([0156](0156-boot-loader-checks-image.md)), in the firmware's own format ([0204](0204-hybrid-signatures.md)). The machine keeps no version of its own, so older signed images still boot ([0209](0209-no-rollback-protection.md)).

## Options

1. **An offline root key that delegates.** It signs statements naming the working keys, and the boot loader carries only the root's public key.
2. **One key per role,** each embedded where it's checked.
3. **One project key** that signs everything.

## Decision

Option 1.

- The project keeps a root key offline. It signs statements that name the working keys: one for system images, one for boot loaders, and one for packages the project publishes outside the image.
- Each statement has a version. A newer statement can replace or revoke a working key.
- The boot loader carries only the root's public key. A system image comes with the statement that names its key.
- Revoking or replacing a working key ships a new boot loader, which refuses statements older than the one that revoked it ([0208](0208-boot-loader-beside-old.md)).
- Owners who build their own images use a root key of their own, and build their own boot loader with it.
- The boot loader carries two signatures. Secure Boot checks one, in the firmware's format, against a key the owner enrolls. The other is our own pair ([0204](0204-hybrid-signatures.md)), which the update service checks before it installs a new boot loader.

## Consequences

- **Losing a working key doesn't lose the root.** A new statement replaces it.
- **The root key is used rarely,** only to sign statements, so it can stay on a machine that's never online.
- **A revoked key stays usable by an offline attacker** on a machine where the old boot loader can still be put back ([0202](0202-offline-attackers.md), [0209](0209-no-rollback-protection.md)).
- **Secure Boot needs the owner to enroll a key** at first. A signature from Microsoft's UEFI certificate authority, which most PCs trust out of the box, can come later ([08-roadmap](../design/08-roadmap.md)).
- **Signing happens apart from the build.** Builds are reproducible, so the key holder signs what anyone can rebuild ([0194](0194-reproducible-builds.md)).
- **02's question on signing keys is answered, and so is 0156's on who signs the system image and how the boot loader knows its key.**
