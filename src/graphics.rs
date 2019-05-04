use crate::gfx_types::*;
use crate::res::TextureAssets;
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

    #[inline]
    pub fn factory(&self) -> &Factory {
        &self.factory
    }

    #[inline]
    pub fn factory_mut(&mut self) -> &mut Factory {
        &mut self.factory
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

    pub fn create_texture_cache() -> TextureAssets {
        TextureAssets::new()
    }
}

pub type GlTextureAssets = TextureAssets;
