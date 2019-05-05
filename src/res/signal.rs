//! Module for signal pump.
//!
//! ```ignore
//!      Main Loop                     Signal Hub
//!          |                              |
//! Buffer B |>--------- Signal 1 --------->|
//!          |>--------- Buffer A --------->|
//!          |                              | Dispatch Signal 1
//!          |                              | Publish Signal 2
//!          |                              | Publish Signal 3
//!          |         [ Signal 2 ]         |
//!          |<--------[ Signal 3 ]--------<|
//!          |         [ Buffer A ]         |
//! Buffer A |                              |
//!          |>--------- Signal 2 --------->|
//!          |>--------- Buffer B --------->|
//!          |                              | Dispatch Signal 2
//!          |                              |
//!          v                              v
//! ```

// TODO: Rename to Message and use simple crossbeam channel

use core::marker::PhantomData;
use crossbeam::channel::{bounded, Receiver, Sender};
use specs::{RunNow, System};
use std::any::TypeId;
use std::collections::{HashSet, VecDeque};
use std::mem;

/// Registry of slots that subscribe to
/// specified events.
pub struct SignalHub<'a, K: SignalKey> {
    systems: Vec<Box<dyn RunNow<'a>>>,
    _marker: PhantomData<K>,
}

impl<'a, K> SignalHub<'a, K>
where
    K: SignalKey,
{
    pub fn new() -> Self {
        SignalHub {
            systems: vec![],
            _marker: PhantomData,
        }
    }

    pub fn register<T>(&mut self, signal_key: K, handler: T)
    where
        T: System<'a>,
    {
        unimplemented!()
    }

    pub fn dispatch(&mut self) {
        unimplemented!()
    }
}

impl<'a, K> Default for SignalHub<'a, K>
where
    K: SignalKey,
{
    fn default() -> SignalHub<'a, K> {
        SignalHub {
            systems: vec![],
            _marker: PhantomData,
        }
    }
}

pub trait SignalKey {
    fn key() -> usize;
}

/// Event that is scheduled to be emitted.
///
/// Keeps track of which system types scheduled
/// the event, to avoid delivering an event to
/// it's sender.
pub enum ScheduledSignal<T> {
    /// Simple untracked signal.
    SimpleSignal,

    /// Kept separate so that simple cases don't
    /// incure set allocations.
    TrackedSignal { signal: T, senders: HashSet<TypeId> },
}

pub type SignalBuffer<T> = VecDeque<ScheduledSignal<T>>;

pub struct SignalPublish<T> {
    buffer: SignalBuffer<T>,
}

impl<T> SignalPublish<T> {
    pub fn new(buffer: SignalBuffer<T>) -> Self {
        SignalPublish { buffer }
    }

    pub fn swap(&mut self, mut buffer: &mut SignalBuffer<T>) {
        mem::swap(&mut self.buffer, &mut buffer);
    }
}

/// Bi-directional channel to send signal
/// buffer across thread boundries.
pub struct SignalChannel<T> {
    sender: Sender<SignalBuffer<T>>,
    receiver: Receiver<SignalBuffer<T>>,
}

impl<T> SignalChannel<T> {
    /// Create a pair of bi-directional signal channels
    /// to be used on either side of a thread boundry.
    pub fn pair() -> (Self, Self) {
        let (a_send, a_recv) = bounded(2);
        let (b_send, b_recv) = bounded(2);

        (
            SignalChannel {
                sender: a_send,
                receiver: b_recv,
            },
            SignalChannel {
                sender: b_send,
                receiver: a_recv,
            },
        )
    }
}
