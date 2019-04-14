use crate::gfx_types::*;
use crossbeam::channel::{Receiver, RecvError, SendError, Sender};
use gfx::format::Formatted;
use gfx::Encoder;
use gfx_core::handle::{DepthStencilView, RenderTargetView};
use gfx_core::memory::Typed;
use gfx_device::{CommandBuffer, Device, Factory, Resources};
use glutin::WindowedContext;

/// Wrapper for Glutin objects
///
/// TODO: Move into specs resources
#[allow(dead_code)]
pub struct GraphicContext {
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

    /// Update the internal dimensions of the main framebuffer targets
    ///
    /// Essentially implements the `update_views` function from `gfx_window_glutin`.
    ///
    /// see [Function gfx_window_glutin::update_views](https://docs.rs/gfx_window_glutin/0.30.0/gfx_window_glutin/fn.update_views.html)
    ///
    /// Anything that cloned the handle to either the render target or depth stencil
    /// will have to retrieve new handles. Internally the function creates new buffers
    /// and thus the references are not longer shared.
    pub fn update_views(&mut self) {
        let dim = self.render_target.get_dimensions();
        assert_eq!(dim, self.depth_stencil.get_dimensions());
        if let Some((cv, dv)) = gfx_window_glutin::update_views_raw(
            &self.window,
            dim,
            ColorFormat::get_format(),
            DepthFormat::get_format(),
        ) {
            self.render_target = Typed::new(cv);
            self.depth_stencil = Typed::new(dv);
        }
    }
}

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

    pub fn send_block(&mut self, encoder: Encoder<R, C>) -> Result<(), SendError<Encoder<R, C>>> {
        self.send.send(encoder)
    }

    pub fn recv_block(&mut self) -> Result<Encoder<R, C>, RecvError> {
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
