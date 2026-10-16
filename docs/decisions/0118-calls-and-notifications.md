# 0118. IPC is synchronous calls through endpoints, with notifications for events

- **Status:** Accepted
- **Date:** 2026-10-17

## Context

[0021](0021-ipc-simple-upgradable.md) left the first IPC design to Day 5. Calls that a service replies to are already required: time and priority travel with a call and come back with the reply ([0085](0085-clients-pay-cpu-time.md)), and the core passes straight between client and service ([0091](0091-direct-switch-on-calls.md)). Some events aren't calls, because nobody should wait for them to be handled: interrupts for drivers, memory notices ([0076](0076-memory-notice-deadline.md)) and reports to supervisors ([0104](0104-supervisor.md)).

## Options

1. **Calls and notifications.** Synchronous call and reply through endpoints, as in L4 and seL4, plus notification objects.
2. **Calls and asynchronous channels.** Calls as in option 1, plus Zircon-style channels whose messages queue in kernel objects the sender pays for.
3. **Asynchronous channels only,** as in Zircon.

## Decision

Option 1.

- **Endpoints carry calls.** A client calls through an endpoint capability and waits. The service receives the message, and its reply goes back to the client.
- **Notifications carry events.** A notification is a word of bits. Any holder of a capability to it can set bits without blocking, and one thread waits for them and collects them.
- Bulk data goes through shared regions ([0058](0058-shared-and-handed-over.md)).
- The kernel never queues messages. A caller whose service isn't ready waits in its own thread object.

## Consequences

- **The kernel gains three IPC object types:** endpoints, notifications and reply objects ([0120](0120-reply-objects.md)). Programs pay for them like other kernel objects ([0026](0026-programs-pay-kernel-objects.md)).
- **Senders of events never wait for receivers.** A slow or hostile program can't hold up the memory service or a driver's interrupts ([0016](0016-threat-model-scope.md)).
- **Events carry no data beyond their bits.** A receiver that needs details calls the sender ([0132](0132-events-notify-then-call.md)).
- **Interrupts can reach drivers as notifications.** The driver model decides ([06-system](../design/06-system.md)).
- **Channels can be added later as an upgrade** ([0021](0021-ipc-simple-upgradable.md)), behind the system library ([0117](0117-system-library.md)).
- **05's question on the first IPC design is answered,** together with [0119](0119-message-eight-words.md) to [0124](0124-large-calls-split.md).
