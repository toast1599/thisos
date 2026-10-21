# 0279. Progress lives in a dated log, in docs/progress.md

- **Status:** Accepted
- **Date:** 2026-10-20

## Context

08-roadmap marks each step's status, and git keeps the commits ([0228](0228-step-start-and-done.md)). Nothing records what a step designed, added or did, or the evidence for its done conditions ([0272](0272-roadmap-reopened.md)).

## Options

1. **A dated log in the repository.** As for its shape:
   - **One file, appended.**
   - **A file for each working day,** with an index.
   - **A file for each milestone.**
2. **The step table, with evidence,** and no log.
3. **GitHub issues and a project board.**

## Decision

Option 1, one file, appended.

- `docs/progress.md` gets a dated section for each working day. Each section has the same four parts:
  - **Designed:** the records written;
  - **Added:** the paths changed;
  - **Done:** each work item, step or milestone finished, with its evidence: CI runs, reviews, test counts;
  - **Next:** what follows.
- 08-roadmap's step and milestone tables link to the sections.

## Consequences

- **Progress is recorded by day,** in the repository, beside what it describes.
- **Each Done names its evidence,** so anyone can check it.
