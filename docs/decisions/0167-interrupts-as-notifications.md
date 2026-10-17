# 0167. Interrupts reach drivers as notifications

- **Status:** Accepted, amended by [0214](0214-interrupt-remapping.md)
- **Date:** 2026-10-17

## Context

Drivers run in user mode ([0014](0014-microkernel.md)). Events reach programs as notifications, and the kernel never queues messages ([0118](0118-calls-and-notifications.md)). 0118 left open whether interrupts reach drivers as notifications.

## Options

1. **Interrupt capabilities bound to notifications,** as in seL4.
2. **Messages to an endpoint.**
3. **Interrupt objects** that a driver waits on, as in Zircon.

## Decision

Option 1.

- The root task holds an interrupt-control capability ([0159](0159-root-task-boot-information.md)). With it, a capability is made for one interrupt line or one MSI vector, and bound to a notification.
- When the interrupt arrives, the kernel masks it and signals the notification.
- The driver acknowledges through the interrupt capability, which unmasks it.
- For an MSI vector, the kernel gives the address and data value that the device writes.

## Consequences

- **The kernel gains one object type,** the interrupt capability, and no new kind of wait ([0118](0118-calls-and-notifications.md)).
- **One driver thread can serve calls and interrupts,** with the notification bound to it ([0122](0122-bound-notifications.md)).
- **An interrupt can't arrive again until it's acknowledged,** so a slow driver holds up only its own device.
- **The kernel programs the interrupt controller** to mask and route interrupts, as part of the platform code 0032 allows.
- **0118's question on interrupts is answered.**
