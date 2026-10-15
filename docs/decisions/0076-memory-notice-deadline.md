# 0076. Programs over their fair share get a notice, then a deadline

- **Status:** Accepted
- **Date:** 2026-10-15

## Context

Under [0049](0049-memory-fair-share.md), when memory is short, programs over their fair share are asked to give memory back. If they don't, their memory is paged out or they are stopped. How they're asked, and how long they have, was open.

## Options

1. **A notice, then a deadline.**
2. **A notice and a deadline, plus discardable memory** that the memory service can take back at once without asking.
3. **No notice.** Memory is paged out, or the program stopped, straight away.

## Decision

Option 1.

- The memory service sends each program over its share a notice saying how much memory to give back.
- If a program hasn't given back enough by a deadline, its memory is paged out or it is stopped ([0049](0049-memory-fair-share.md)).
- The deadline is a setting in the memory service's policy, not fixed by the design.

## Consequences

- **Caches can shrink before anything is stopped.**
- **How the notice is sent comes with the IPC design** ([05-interfaces](../design/05-interfaces.md)).
- **Discardable memory can be added later** without breaking anything.
- **03's question on asking for memory back is answered.**
