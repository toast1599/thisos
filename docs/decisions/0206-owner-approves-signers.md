# 0206. A program runs if the system image's signer or a signer the owner approved signed it

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

Programs carry a signature, and the device owner's policy decides which signers' programs may run ([0040](0040-signed-programs-owner-policy.md)). 0040 left the default policy open ([02-verification](../design/02-verification.md)). The system image is checked at boot ([0156](0156-boot-loader-checks-image.md)). The login service gives each session an identity capability, and services that decide access by their own policy may act on it ([0141](0141-identity-capability.md)).

## Options

1. **The image's signer, and signers the owner approves.**
2. **Any valid signature,** with signers the owner blocks refused.
3. **The project's store only,** with others added by the owner.

## Decision

Option 1.

- Packages signed by the system image's signer always run.
- A package from any other signer runs only once the owner has approved that signer. Approvals can be withdrawn.
- A developer's own builds run once the owner approves the developer's key, which the build tool signs with.
- The package store keeps the approvals, and accepts changes from the owner.

## Consequences

- **Nothing runs from an unknown signer** without the owner's say.
- **Approving a signer lets its programs run, and grants them nothing.** What a program can reach still comes only from the capabilities it's given ([0020](0020-capabilities.md)).
- **The package store recognises the owner by identity** ([0141](0141-identity-capability.md)). It decides by its own policy, so it may act on identity.
- **An offline attacker can change the approvals** until stored data is protected ([0202](0202-offline-attackers.md)).
- **0040's question on the default policy is answered.**
