# 0130. A thread's pager and exception handler can be changed through its control right

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Each thread names its pager ([0081](0081-pager-per-thread.md)) and its exception handler ([0102](0102-exception-handler.md)) when it's made. Both records, and [0056](0056-pager-chosen-at-spawn.md), left open whether these can change later, and who may change them. Debuggers attach to running programs, and a language runtime may want to take over paging after it starts ([0056](0056-pager-chosen-at-spawn.md)).

## Options

1. **By a holder of the control right,** at any time.
2. **Fixed at creation.**

## Decision

Option 1.

- Any holder of a thread's capability with the control right ([0128](0128-rights-and-badges.md)) can change the thread's pager or its exception handler at any time.
- A program normally holds that right for its own threads ([0108](0108-startup-information.md)).

## Consequences

- **A language runtime can install its own pager thread** after startup ([0056](0056-pager-chosen-at-spawn.md)), and **a debugger can attach** to a running program.
- **A spawner can keep a program's threads on its pager** by handing over their capabilities without the control right. Threads the program makes itself are its own to set.
- **A program can move its exception handler away from its supervisor.** Its supervisor then no longer sees its crashes, but the program harms only itself, and its supervisor can still take its memory back ([0105](0105-supervisor-reclaims.md)).
- **0056's, 0081's and 0102's questions on changing handlers are answered.**
