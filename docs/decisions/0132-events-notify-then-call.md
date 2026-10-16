# 0132. An event is a notification, followed by a call for its details

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

Some events have to reach programs without the sender waiting: the memory service's notice to give memory back ([0076](0076-memory-notice-deadline.md)), the report to a supervisor that its program was stopped ([0104](0104-supervisor.md), [0105](0105-supervisor-reclaims.md)), and threads the kernel suspended because their fault had nowhere to go ([0103](0103-fault-without-handler.md)). Notifications carry only bits ([0118](0118-calls-and-notifications.md)).

## Options

1. **A notification, then a call.** The sender sets a bit, and the receiver calls back for the details.
2. **Calls into the receiver.** The sender calls an endpoint that the receiver serves.

## Decision

Option 1.

- A sender sets a bit in a notification the receiver gave it. Setting it never blocks.
- The receiver calls the sender for the details, such as how much memory to give back, or how a program ended.
- A program that wants memory notices gives the memory service a notification through its memory capability ([0137](0137-memory-capability-per-program.md)). A supervisor does the same through its reclaim capability ([0105](0105-supervisor-reclaims.md)), to hear when its program is stopped.
- For threads whose fault has nowhere to go, each address space names a notification, which the kernel signals when it suspends one of its threads ([0103](0103-fault-without-handler.md)). It's named when the address space is made, normally one the supervisor holds, and can be changed through the control right ([0128](0128-rights-and-badges.md)).

## Consequences

- **No program can hold up the memory service** ([0016](0016-threat-model-scope.md)). The service never waits for a receiver.
- **A program that doesn't watch for notices** meets the deadline without warning ([0076](0076-memory-notice-deadline.md)).
- **A supervisor notices suspended threads** without the kernel knowing about supervisors ([0104](0104-supervisor.md)).
- **Crashes reach the supervisor as fault messages** when it's the program's exception handler ([0102](0102-exception-handler.md)).
- **A program's report of its own exit status stays a call** to its supervisor ([0104](0104-supervisor.md)), since the program is the one that waits.
- **The details are typed in the IDL** like any other interface ([0142](0142-own-idl.md)).
- **05's question on memory notices is answered, and so are 0103's and 0104's on how a supervisor is told.**
