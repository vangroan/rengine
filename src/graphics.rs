use crate::gfx_types::*;
use crossbeam::channel::{Receiver, RecvError, SendError, Sender};
use gfx::Encoder;
use gfx_core::handle::{DepthStencilView, RenderTargetView};
use gfx_device::{CommandBuffer, Device, Factory, Resources};
use glutin::{EventsLoop, WindowedContext};

/// Wrapper for Glutin objects
///
/// TODO: Move into specs resources
#[allow(dead_code)]
pub struct GraphicContext {
    pub(crate) events_loop: EventsLoop,
    pub(crate) window: WindowedContext,
    pub(crate) device: Device,
    pub(crate) factory: Factory,
    pub(crate) render_target: RenderTargetView<Resources, ColorFormat>,
    pub(crate) depth_stencil: DepthStencilView<Resources, DepthFormat>,
}

impl GraphicContext {
    #[inline]
    pub fn window(&self) -> &WindowedContext {
        &self.window
    }

    #[inline]
    pub fn window_mut(&mut self) -> &mut WindowedContext {
        &mut self.window
    }

    pub fn create_encoder(&mut self) -> Encoder<Resources, CommandBuffer> {
        self.factory.create_command_buffer().into()
    }
}

/// Channels for sending graphics encoders accross thread boundries
#[derive(Clone)]
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

    pub fn send_block(&mut self, encoder: Encoder<R, C>) -> Result<(), SendError<Encoder<R, C>>> {
        self.send.send(encoder)
    }

    pub fn recv_block(&mut self) -> Result<Encoder<R, C>, RecvError> {
        self.recv.recv()
    }
}
