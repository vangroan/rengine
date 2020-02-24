/// Tools for inter-thread communication.
pub use channel::{RecvError, SendError};
use crossbeam::channel;

/// A pair of multiple-producer-multiple-consumer channels
/// for bidrectional communication between threads.
///
/// For when two threads need to pass ownership of a value
/// between each other, waiting for the other to finish before
/// continuing.
///
/// A common use case to pass around command buffers between
/// contexts that are not thread-safe.
///
/// ```
/// extern crate rengine;
///
/// use rengine::sync::ChannelPair;
/// use std::thread;
///
/// let (mut chan_a, mut chan_b): (ChannelPair<Vec<u32>>, ChannelPair<Vec<u32>>) = ChannelPair::create();
///
/// let thread_a = thread::spawn(move || {
///     let buf = vec![];
///
///     // Waits for thread_b to receive buffer.
///     chan_a.send(buf).unwrap();
///
///     // Waits for thread_b to send buffer back.
///     let buf = chan_a.receive().unwrap();
///
///     // Execute commands in buffer.
///     assert!(buf[0] == 1);
///     assert!(buf[1] == 2);
///     assert!(buf[2] == 3);
/// });
///
/// let thread_b = thread::spawn(move || {
///     let mut buf = chan_b.receive().unwrap();
///
///     // Commands determined by thread_b but need to be
///     // executed by thread_a.
///     buf.push(1);
///     buf.push(2);
///     buf.push(3);
///
///     chan_b.send(buf).unwrap();
/// });
///
/// thread_a.join();
/// thread_b.join();
///
/// ```
pub struct ChannelPair<T: Send> {
    sender: channel::Sender<T>,
    receiver: channel::Receiver<T>,
}

impl<T: Send> ChannelPair<T> {
    /// Creates two channel pairs, that are linked to eachother.
    pub fn create() -> (Self, Self) {
        // Bounded channel with no capacity  will block on both
        // send and receive, until the other end is ready.
        let (a_send, b_recv): (channel::Sender<T>, channel::Receiver<T>) = channel::bounded(0);
        let (b_send, a_recv): (channel::Sender<T>, channel::Receiver<T>) = channel::bounded(0);

        let a = ChannelPair {
            sender: a_send,
            receiver: a_recv,
        };
        let b = ChannelPair {
            sender: b_send,
            receiver: b_recv,
        };

        (a, b)
    }

    pub fn send(&mut self, val: T) -> Result<(), SendError<T>> {
        self.sender.send(val)
    }

    pub fn receive(&mut self) -> Result<T, RecvError> {
        self.receiver.recv()
    }
}

/// Implicit implementation via derive doesn't work.
impl<T: Send> Clone for ChannelPair<T> {
    fn clone(&self) -> Self {
        ChannelPair {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}
