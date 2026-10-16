# 0102. Each thread names an exception handler besides its pager

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

Each thread names a pager for its page faults ([0081](0081-pager-per-thread.md)). Whether its other exceptions, such as division by zero, an invalid instruction or a breakpoint, go to the same handler was open. The kernel never writes program memory ([0072](0072-registers-only.md)), so it can't build a signal frame on a thread's stack, as Unix does.

## Options

1. **A second handler.** Each thread names an exception handler besides its pager.
2. **The pager handles everything,** as in seL4, and forwards what isn't about memory.
3. **Stop the program** on any exception other than a page fault.

## Decision

Option 1.

- Each thread names an exception handler when it's created, besides its pager. Examples are its program's supervisor or a debugger.
- Exceptions other than page faults go to the exception handler as a message in registers ([0072](0072-registers-only.md)).
- The handler can resume the thread with changed registers, or stop it.
- Pagers deal only with page faults.

## Consequences

- **Debuggers and language runtimes can handle a program's exceptions.**
- **Pagers stay about memory.** The default pager never deals with crashes or debugging.
- **A handler that needs more than the message carries** reads the thread's registers with further calls, whose shape comes with the interfaces ([0072](0072-registers-only.md), [05-interfaces](../design/05-interfaces.md)).
- **Whether a thread's exception handler can be changed after it's created, and by whom,** is part of the capability design ([05-interfaces](../design/05-interfaces.md)).
