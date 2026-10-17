# 0212. A program can declare the highest address for its image

- **Status:** Accepted
- **Date:** 2026-10-18

## Context

Programs that keep tags in the upper bits of pointers ask for low addresses when they reserve address space ([0078](0078-low-addresses-on-request.md), [0138](0138-pager-keeps-reservations.md)). Their code and static data are placed by the loader ([0061](0061-no-randomization-yet.md)). 0147 left open whether a program can ask for its own image to be placed at low addresses ([02-verification](../design/02-verification.md)).

## Options

1. **Yes, by declaration.**
2. **No.** Only memory reserved at run time can be low.

## Decision

Option 1.

- A program's declarations can give the highest address its image may use. This amends 0147, whose list of declarations gains it.
- The loader places the image below that address.

## Consequences

- **Pointers into code and static data can be tagged too.**
- **The lowest 64 KiB still stays unmapped** ([0074](0074-low-64k-unmapped.md)).
- **Without the declaration, the image can go anywhere** in the lower half ([0063](0063-widest-addressing.md)).
- **Randomizing later still works,** within the limit ([0061](0061-no-randomization-yet.md)).
- **0147's question on placing the image at low addresses is answered.**
