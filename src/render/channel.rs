use crossbeam::channel::{Receiver, RecvError, SendError, Sender};

/// Channels for sending graphics encoders accross thread boundries
pub struct ChannelPair<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    send: Sender<gfx::Encoder<R, C>>,
    recv: Receiver<gfx::Encoder<R, C>>,
}

impl<R, C> ChannelPair<R, C>
where
    R: gfx::Resources,
    C: gfx::CommandBuffer<R>,
{
    pub fn new() -> Self {
        // Thread will block if more than 1 encoder is being sent
        let (send, recv) = crossbeam::channel::bounded(1);
        ChannelPair { recv, send }
    }

    pub fn send_block(
        &mut self,
        encoder: gfx::Encoder<R, C>,
    ) -> Result<(), SendError<gfx::Encoder<R, C>>> {
        self.send.send(encoder)
    }

    pub fn recv_block(&mut self) -> Result<gfx::Encoder<R, C>, RecvError> {
        self.recv.recv()
    }
}

impl<R, C> Clone for ChannelPair<R, C>
where
    R: gfx::Resources,
    C: gfx::CommandBuffer<R>,
{
    fn clone(&self) -> Self {
        ChannelPair {
            send: self.send.clone(),
            recv: self.recv.clone(),
        }
    }
}
