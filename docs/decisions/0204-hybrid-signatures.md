# 0204. Signatures pair Ed25519 with ML-DSA-65

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

Programs are signed ([0040](0040-signed-programs-owner-policy.md)), and the boot loader checks the kernel and the ramdisk against the system image's signature ([0156](0156-boot-loader-checks-image.md)). A system image and a boot loader stay in use for years. ML-DSA is NIST's post-quantum signature standard (FIPS 204, 2024). Checked on 2026-10-18: UEFI Secure Boot checks boot loaders only in the firmware's own format, Authenticode, usually with RSA, and UEFI's post-quantum support is still a plan.

## Options

1. **Ed25519 and ML-DSA-65 together.** Both must verify.
2. **Ed25519 only.**
3. **ECDSA P-256.**

## Decision

Option 1.

- Every signature that thisOS's own code checks is a pair: an Ed25519 signature and an ML-DSA-65 signature. Both must verify.
- Each signature names its algorithms, so that others can be added later.
- That covers packages, system images, boot loaders and the root key's statements ([0205](0205-root-and-working-keys.md)).

## Consequences

- **A signature stays trustworthy if either algorithm falls,** including Ed25519 to a quantum computer.
- **Signatures are larger,** about 3.3 KB each, which is small beside a package.
- **The verifier is trusted.** Its code in the boot loader, the package store and the update service is either free of unsafe code or in the audited set ([0191](0191-audited-list-and-wrapper.md)).
- **Secure Boot's signature on the boot loader stays in the firmware's format** ([0205](0205-root-and-working-keys.md)).
