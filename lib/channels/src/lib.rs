//! A channel's state: two queues of messages, one toward each endpoint, and
//! whether each endpoint is closed. A message carries bytes and handles of
//! type `T`. The kernel adds the lock, the user copies and what a handle
//! is.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::mem;

use thisos_abi::{Error, MAX_QUEUED};

/// One of a channel's two endpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// The endpoint listed first when the channel was made.
    First,
    /// The other one.
    Second,
}

impl Side {
    /// The other endpoint.
    #[must_use]
    pub const fn peer(self) -> Side {
        match self {
            Side::First => Side::Second,
            Side::Second => Side::First,
        }
    }

    const fn index(self) -> usize {
        match self {
            Side::First => 0,
            Side::Second => 1,
        }
    }
}

/// What one send carries.
#[derive(Debug, PartialEq, Eq)]
pub struct Message<T> {
    /// What the sender wrote.
    pub bytes: Vec<u8>,
    /// What moved out of the sender's handle table.
    pub handles: Vec<T>,
}

/// A channel's two queues, each holding at most [`MAX_QUEUED`] messages.
pub struct Channel<T> {
    /// The messages waiting at each endpoint, oldest first.
    queues: [VecDeque<Message<T>>; 2],
    closed: [bool; 2],
}

impl<T> Channel<T> {
    /// A channel with both endpoints open and nothing queued, which
    /// allocates nothing.
    pub const fn new() -> Self {
        Self {
            queues: [VecDeque::new(), VecDeque::new()],
            closed: [false, false],
        }
    }

    /// Queues the message `make` returns at `from`'s peer. Calls `make`
    /// only once the message has a place, so a send that fails with
    /// [`Error::PeerClosed`], [`Error::ShouldWait`] if [`MAX_QUEUED`]
    /// messages wait there, or [`Error::NoMemory`], never calls it. A
    /// failure from `make` sends nothing.
    pub fn send(
        &mut self,
        from: Side,
        make: impl FnOnce() -> Result<Message<T>, Error>,
    ) -> Result<(), Error> {
        let to = from.peer();
        if self.closed[to.index()] {
            return Err(Error::PeerClosed);
        }
        let queue = &mut self.queues[to.index()];
        if queue.len() >= MAX_QUEUED {
            return Err(Error::ShouldWait);
        }
        queue.try_reserve(1).map_err(|_| Error::NoMemory)?;
        queue.push_back(make()?);
        Ok(())
    }

    /// Hands the oldest message waiting at `at` to `take`, and returns what
    /// it returns. A failed `take` hands the message back, and it stays the
    /// oldest. Fails with [`Error::ShouldWait`] if no message waits, or
    /// [`Error::PeerClosed`] if none waits and the peer is closed.
    pub fn receive<R>(
        &mut self,
        at: Side,
        take: impl FnOnce(Message<T>) -> Result<R, (Error, Message<T>)>,
    ) -> Result<R, Error> {
        let queue = &mut self.queues[at.index()];
        let Some(message) = queue.pop_front() else {
            return Err(if self.closed[at.peer().index()] {
                Error::PeerClosed
            } else {
                Error::ShouldWait
            });
        };
        take(message).map_err(|(e, message)| {
            // Popping left its place free.
            queue.push_front(message);
            e
        })
    }

    /// Closes `side`, and returns the messages that waited there, which
    /// nothing can receive now. Messages it sent stay for its peer to
    /// receive. Closing it again returns nothing.
    pub fn close(&mut self, side: Side) -> VecDeque<Message<T>> {
        self.closed[side.index()] = true;
        mem::take(&mut self.queues[side.index()])
    }
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn message(byte: u8, handles: &[char]) -> Message<char> {
        Message {
            bytes: vec![byte],
            handles: handles.to_vec(),
        }
    }

    fn receive(channel: &mut Channel<char>, at: Side) -> Result<Message<char>, Error> {
        channel.receive(at, Ok)
    }

    #[test]
    fn messages_arrive_in_order_at_the_peer() {
        let mut channel = Channel::new();
        channel
            .send(Side::First, || Ok(message(1, &['a'])))
            .unwrap();
        channel.send(Side::First, || Ok(message(2, &[]))).unwrap();
        channel.send(Side::Second, || Ok(message(3, &[]))).unwrap();
        assert_eq!(receive(&mut channel, Side::Second), Ok(message(1, &['a'])));
        assert_eq!(receive(&mut channel, Side::Second), Ok(message(2, &[])));
        assert_eq!(receive(&mut channel, Side::Second), Err(Error::ShouldWait));
        assert_eq!(receive(&mut channel, Side::First), Ok(message(3, &[])));
    }

    #[test]
    fn a_full_queue_fails_without_making_the_message() {
        let mut channel = Channel::new();
        for i in 0..MAX_QUEUED {
            let byte = u8::try_from(i).unwrap();
            channel
                .send(Side::First, || Ok(message(byte, &[])))
                .unwrap();
        }
        let refused = channel.send(Side::First, || panic!("made a message for a full queue"));
        assert_eq!(refused, Err(Error::ShouldWait));
        // The other direction has its own room.
        channel.send(Side::Second, || Ok(message(0, &[]))).unwrap();
        receive(&mut channel, Side::Second).unwrap();
        channel.send(Side::First, || Ok(message(0, &[]))).unwrap();
    }

    #[test]
    fn a_failed_make_sends_nothing() {
        let mut channel = Channel::new();
        let sent = channel.send(Side::First, || Err(Error::AccessDenied));
        assert_eq!(sent, Err(Error::AccessDenied));
        assert_eq!(receive(&mut channel, Side::Second), Err(Error::ShouldWait));
    }

    #[test]
    fn a_failed_take_leaves_the_message_oldest() {
        let mut channel = Channel::new();
        channel
            .send(Side::First, || Ok(message(1, &['a'])))
            .unwrap();
        channel.send(Side::First, || Ok(message(2, &[]))).unwrap();
        let taken = channel.receive(Side::Second, |m| Err::<(), _>((Error::BadAddress, m)));
        assert_eq!(taken, Err(Error::BadAddress));
        assert_eq!(receive(&mut channel, Side::Second), Ok(message(1, &['a'])));
    }

    #[test]
    fn a_closed_peer_is_drained_before_it_fails() {
        let mut channel = Channel::new();
        channel.send(Side::First, || Ok(message(1, &[]))).unwrap();
        channel
            .send(Side::Second, || Ok(message(2, &['b'])))
            .unwrap();
        let dropped = channel.close(Side::First);
        assert_eq!(dropped, [message(2, &['b'])]);
        assert!(channel.close(Side::First).is_empty());
        let refused = channel.send(Side::Second, || panic!("made a message for a closed peer"));
        assert_eq!(refused, Err(Error::PeerClosed));
        assert_eq!(receive(&mut channel, Side::Second), Ok(message(1, &[])));
        assert_eq!(receive(&mut channel, Side::Second), Err(Error::PeerClosed));
    }
}
