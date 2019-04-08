use gfx::Encoder;
use gfx_core::handle::{DepthStencilView, RenderTargetView};
use gfx_device::{CommandBuffer, Device, Factory, Resources};
use glutin::{EventsLoop, WindowedContext};

use crate::gfx_types::*;

/// Wrapper for Glutin objects
#[allow(dead_code)]
pub struct GraphicContext {
    pub(crate) events_loop: EventsLoop,
    pub(crate) encoder: Encoder<Resources, CommandBuffer>,
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
}
