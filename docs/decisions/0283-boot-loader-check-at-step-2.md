# 0283. The boot loader checks the image's signature from step 2

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

The boot loader checks the kernel and the ramdisk against the system image's signature ([0156](0156-boot-loader-checks-image.md)), which pairs Ed25519 with ML-DSA-65 ([0204](0204-hybrid-signatures.md)), through an offline root key's statements of the working keys ([0205](0205-root-and-working-keys.md)). The roadmap built that at step 20, so every image before it was unsigned ([0227](0227-steps-and-phases.md)). Day 8 turned down verified boot early, because it brought forward about 25,000 lines of signature crates to audit. Trusted crates aren't audited ([0285](0285-trusted-publishers.md)).

## Options

1. **Early, with the boot loader.**
2. **With packages,** at step 20, as now.
3. **Signing early, checking late.**

## Decision

Option 1.

- Step 2 builds the boot loader's check: of the kernel and the ramdisk, against the image's signature, through statements of a test root key.
- The signing tools sign every image with test keys from then on.
- The attack tests on a tampered kernel, ramdisk or statement run from step 2.
- Step 20 keeps the try count and the fallback ([0158](0158-image-try-count.md)), the update service, boot loaders installed beside the old one ([0208](0208-boot-loader-beside-old.md)), the refusal of a revoked working key, and Secure Boot in QEMU with an enrolled key ([0205](0205-root-and-working-keys.md)).
- This amends 0227, whose images are signed and checked from step 2.

## Consequences

- **Every milestone image is signed and checked,** from m1.
- **The release's root key, made offline at step 24, replaces the test keys** ([0205](0205-root-and-working-keys.md)).
- **Secure Boot of the boot loader itself still comes at step 20,** so until then the chain starts at the boot loader.
