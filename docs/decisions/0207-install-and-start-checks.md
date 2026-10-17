# 0207. The package store checks packages at install and after each boot, and the loader checks the policy at each start

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

The boot loader checks the system image, and the package store checks the image's other packages by hash ([0156](0156-boot-loader-checks-image.md)). For programs installed later, 0040 and 0156 left open whether the loader or a separate service checks them ([02-verification](../design/02-verification.md)). Every block of a package is checked against its hash as it's read ([0203](0203-package-format.md)), and the owner's approvals of signers can be withdrawn ([0206](0206-owner-approves-signers.md)). Offline attackers are in scope, so records the package store keeps on disk could have been changed ([0202](0202-offline-attackers.md)).

## Options

1. **The store at install, and the loader at each start.**
2. **The loader at every start,** with the full signature.
3. **The store at install only.**

Since the store's records could have been changed offline:

- **Check again after each boot.**
- **Trust the records** until stored data is protected.

## Decision

Option 1, with the store checking again after each boot.

- When a package is installed, the package store checks its signature ([0204](0204-hybrid-signatures.md)) and the owner's policy ([0206](0206-owner-approves-signers.md)).
- The first time it opens a package after each boot, it checks the manifest's signature again. It trusts none of its own records on disk by themselves.
- It checks every block by hash as it reads it ([0203](0203-package-format.md)).
- At each start, the loader checks that the owner's policy still allows the package's signer.

## Consequences

- **A withdrawn approval takes effect at the next start.**
- **Tampering with the store's disk is caught,** by the manifest's signature and the blocks' hashes, at the next boot.
- **The cost is one signature check per package per boot,** on a manifest of a few kilobytes.
- **The loader needs no signature code** of its own.
- **02's question on where programs installed later are checked is answered.**
