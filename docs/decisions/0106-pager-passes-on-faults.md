# 0106. A pager passes faults it won't resolve to the thread's exception handler

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

Some page faults can't be resolved by mapping memory: a null access in the low 64 KiB ([0074](0074-low-64k-unmapped.md)), a stack that can't grow ([0048](0048-reserve-on-grant.md)), and memory that its payer took back ([0065](0065-lost-memory-is-an-error.md)). They reach the thread's pager ([0081](0081-pager-per-thread.md)), while what to do about a failing thread is its exception handler's business ([0102](0102-exception-handler.md)).

## Options

1. **The pager passes it on** to the thread's exception handler.
2. **The pager decides alone,** stopping the thread or resuming it elsewhere.

## Decision

Option 1.

- A pager's reply to a fault can say it won't map anything.
- The kernel then sends the fault to the thread's exception handler, which decides whether to stop the thread or resume it elsewhere.
- Stack overflow, null accesses and memory taken back are all handled this way.

## Consequences

- **Pagers need no crash policy.** The default pager only maps memory, or declines to.
- **0065's error reaches the code through the exception handler,** which can resume the thread at a recovery point. How the handler finds that point comes with the interfaces ([05-interfaces](../design/05-interfaces.md)).
- **One fault can cost two handler trips:** one to the pager and one to the exception handler.
