# 0003. IPC through kernel-buffered async channels, with shared memory for bulk data

- **Status:** Provisional, until the milestone 4 prototype confirms it
- **Date:** 2026-11-04

## Context

The main alternatives are:

- **L4/seL4-style synchronous call and reply.** Small messages pass in registers, the kernel buffers nothing, and it switches straight to the server. It is the fastest per call, but every server needs threads or extra notifications to handle concurrency.
- **Zircon-style channels.** The kernel queues messages. Services are easier to write, at the cost of a copy and kernel memory per queued message.

The kernel has a heap ([0001](0001-kernel-scope.md)), so buffering messages is cheap to implement.

## Decision

- **Channels.** A channel is a bounded, two-way pair of endpoints whose messages carry bytes and handles ([0002](0002-handles.md)). Sending a handle moves it.
- **Waiting.** A thread can wait on several objects at once.
- **Bulk data.** Large data goes through shared memory objects mapped by both sides, not through channel messages.

## Consequences

- The milestone 4 prototype has to measure the cost of a round trip, so the bounds and message size limits can be set from real numbers. Its results either confirm this record or replace it.
- A full channel has to block the sender or fail the send. The prototype settles which.
- Interrupts reach user space as something a thread can wait on, alongside channels.
