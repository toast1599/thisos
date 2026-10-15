# 0041. Installed programs are immutable packages named by their content hash

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Installed programs need storage that resists tampering. Fuchsia and Nix store software as immutable packages, each named by a hash of its content.

## Options

1. **Immutable packages** named by content hash.
2. **Ordinary files,** protected by the filesystem service's policy ([0030](0030-filesystem-decides-access.md)).

## Decision

Option 1. Installed programs are stored as immutable packages, each named by the hash of its content.

## Consequences

- **Tampering can't go unnoticed.** Changing a package's content changes its name.
- **Versions coexist.** Several versions of a program can be installed side by side, which also makes rollback simple ([0042](0042-updates.md)).
- **Several details are open** ([02-verification](../design/02-verification.md)):
  - the hash function;
  - the package layout;
  - the service that stores packages.
- **The package store is an OS service.** It ships in the system image, so [0017](0017-unsafe-code-kernel-and-services.md) applies to it ([0044](0044-system-image-is-os-shipped.md)).
