# 0203. A package is a signed manifest and files named by SHA-256 Merkle roots

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

Installed programs are immutable packages named by the hash of their content ([0041](0041-immutable-packages.md)). The signature and the package metadata live in the package around the image ([0147](0147-elf-declarations-only.md)). 0041 left open the hash function and the package layout ([02-verification](../design/02-verification.md)). The default pager maps package pages from the package store's cache, and reads them back from storage after dropping them under memory pressure ([0184](0184-shared-file-pages.md)). Working keys can be replaced, so packages may need signing again ([0205](0205-root-and-working-keys.md)). Fuchsia names each file in a package by a Merkle tree's root, and checks each block as it reads it. SHA-256 is hardware-accelerated on both architectures.

## Options

1. **A SHA-256 Merkle tree for each file, and a manifest.**
2. **The same layout, with BLAKE3's own tree mode.**
3. **One archive per package,** named by its SHA-256 hash and checked whole at install.

For the signature:

- **Beside the manifest,** outside the hash that names the package.
- **Inside the manifest,** so the name covers it too.

## Decision

Option 1, with the signature beside the manifest.

- A package is a manifest and a set of content files.
- Each content file is named by the root of a SHA-256 Merkle tree over its 4 KiB blocks.
- The manifest lists the content files by name and hash, and carries the package metadata ([0147](0147-elf-declarations-only.md)).
- The package is named by its manifest's hash.
- The signature ([0204](0204-hybrid-signatures.md)) covers the manifest, and sits beside it, outside the hash that names the package.
- The package store checks every block against its tree as it reads it, including blocks it reads back later.

## Consequences

- **Tampering is caught at the changed block,** whenever it's read, not only at install.
- **A package can be signed again without changing its name.** The signature covers the manifest, and so the contents and the name.
- **Files shared between packages are stored once,** since a file's name is its hash.
- **Pages can be read as they're needed,** each checked on its own ([0184](0184-shared-file-pages.md)).
- **A larger page is checked as several blocks** ([0069](0069-all-page-sizes.md)).
- **The image's list of packages names each by its manifest's hash** ([0155](0155-ramdisk-reaches-storage.md), [0156](0156-boot-loader-checks-image.md)).
- **02's questions on the hash function and the package layout are answered.**
