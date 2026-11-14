//! Channels: two endpoints sharing one [`Channel`] under one lock, so a
//! send and its peer's close never race. An endpoint closes when the last
//! hold on it goes: its one handle, or the message carrying it.

use alloc::sync::Arc;

use spin::Mutex;
use thisos_abi::{Error, Rights};
use thisos_channels::{Channel, Message, Side};

use crate::process::Object;

/// The rights a new endpoint's handle has.
pub const ENDPOINT_RIGHTS: Rights = Rights::READ.union(Rights::WRITE).union(Rights::TRANSFER);

/// What a message carries for each handle sent.
pub type Carried = (Object, Rights);

pub struct Endpoint {
    side: Side,
    /// Taken with interrupts on, and never by a trap handler: sending and
    /// receiving allocate, and take a process's handle-table lock inside
    /// it.
    channel: Arc<Mutex<Channel<Carried>>>,
}

/// A new channel's two endpoints.
pub fn create() -> (Endpoint, Endpoint) {
    let channel = Arc::new(Mutex::new(Channel::new()));
    let first = Endpoint {
        side: Side::First,
        channel: Arc::clone(&channel),
    };
    let second = Endpoint {
        side: Side::Second,
        channel,
    };
    (first, second)
}

impl Endpoint {
    /// Whether `other` is this endpoint or its peer.
    pub fn same_channel(&self, other: &Endpoint) -> bool {
        Arc::ptr_eq(&self.channel, &other.channel)
    }

    /// Sends the message `make` returns to the peer; see [`Channel::send`].
    /// `make` runs under the channel's lock, so it must drop no object.
    pub fn send(
        &self,
        make: impl FnOnce() -> Result<Message<Carried>, Error>,
    ) -> Result<(), Error> {
        self.channel.lock().send(self.side, make)
    }

    /// Hands the oldest message waiting here to `take`; see
    /// [`Channel::receive`]. `take` runs under the channel's lock, so it
    /// must drop no object.
    pub fn receive<R>(
        &self,
        take: impl FnOnce(Message<Carried>) -> Result<R, (Error, Message<Carried>)>,
    ) -> Result<R, Error> {
        self.channel.lock().receive(self.side, take)
    }
}

impl Drop for Endpoint {
    /// Closes this side and drops what waited here. Endpoints in those
    /// messages that nothing else holds close too, and what waited at them
    /// joins the same queue, so a chain of endpoints each queued at the
    /// last drops in a loop, not by recursing once per link.
    fn drop(&mut self) {
        let mut queued = self.channel.lock().close(self.side);
        while let Some(message) = queued.pop_front() {
            for (object, _) in message.handles {
                if let Object::Channel(endpoint) = object
                    && let Some(endpoint) = Arc::into_inner(endpoint)
                {
                    let mut more = endpoint.channel.lock().close(endpoint.side);
                    queued.append(&mut more);
                    // Its own drop now finds nothing queued.
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use thisos_hal::Platform;

    use super::*;
    use crate::Arch;

    fn carry(endpoint: Endpoint) -> Message<Carried> {
        Message {
            bytes: Vec::new(),
            handles: vec![(Object::Channel(Arc::new(endpoint)), ENDPOINT_RIGHTS)],
        }
    }

    #[test_case]
    fn closing_an_endpoint_drops_what_it_held() {
        let allocated = Arch::heap_stats().allocated;
        let (a, b) = create();
        let (c, d) = create();
        assert!(a.send(|| Ok(carry(c))).is_ok());
        drop(a);
        // b still receives what a sent.
        let Ok(message) = b.receive(Ok) else {
            panic!("b lost a's message");
        };
        let [(Object::Channel(c), _)] = &message.handles[..] else {
            panic!("the message lost its endpoint");
        };
        assert!(c.send(|| Ok(carry(create().0))).is_ok());
        drop(message);
        // d drains what c sent before it closed.
        assert!(d.receive(Ok).is_ok());
        assert!(matches!(d.receive(Ok), Err(Error::PeerClosed)));
        assert!(matches!(b.receive(Ok), Err(Error::PeerClosed)));
        drop((b, d));
        assert_eq!(Arch::heap_stats().allocated, allocated);
    }

    #[test_case]
    fn a_long_chain_of_queued_endpoints_drops_without_recursing() {
        let allocated = Arch::heap_stats().allocated;
        // Each new channel's first endpoint holds the chain so far at its
        // second, which is all that holds the second.
        let (mut head, tail) = create();
        drop(tail);
        for _ in 0..2000 {
            let (first, second) = create();
            assert!(first.send(|| Ok(carry(head))).is_ok());
            drop(first);
            head = second;
        }
        drop(head);
        assert_eq!(Arch::heap_stats().allocated, allocated);
    }
}
