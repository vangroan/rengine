use crate::comp::{Mesh, Transform};
use crate::gfx_types;
use crate::graphics::GraphicContext;
use crate::render::Material;
use gfx;
use gfx::format::{ChannelTyped, Formatted};
use gfx::handle;
use gfx::Factory;
use gfx_device::Resources;

pub struct Canvas {
    render_texture: handle::Texture<Resources, <gfx_types::ColorFormat as Formatted>::Surface>,
    depth_texture: handle::Texture<Resources, <gfx_types::DepthFormat as Formatted>::Surface>,
    render_target: handle::RenderTargetView<Resources, gfx_types::ColorFormat>,
    depth_target: handle::DepthStencilView<Resources, gfx_types::DepthFormat>,
}

impl Canvas {
    pub fn new(
        graphic_context: &mut GraphicContext,
        width: u16,
        height: u16,
    ) -> Result<Canvas, gfx::CombinedError> {
        let (render_texture, render_target) =
            Canvas::create_render(graphic_context, [width, height])?;
        let (depth_texture, depth_target) = Canvas::create_depth(graphic_context, [width, height])?;

        Ok(Canvas {
            render_texture,
            render_target,
            depth_texture,
            depth_target,
        })
    }

    fn create_render(
        graphic_context: &mut GraphicContext,
        size: [u16; 2],
    ) -> Result<
        (
            handle::Texture<Resources, <gfx_types::ColorFormat as Formatted>::Surface>,
            handle::RenderTargetView<Resources, gfx_types::ColorFormat>,
        ),
        gfx::CombinedError,
    > {
        // Texture settings
        let kind = gfx::texture::Kind::D2(size[0], size[1], gfx::texture::AaMode::Single);
        let levels = 1;
        // Shader resource is required, otherwise render target is unsupported
        let bind = gfx::memory::Bind::SHADER_RESOURCE | gfx::memory::Bind::RENDER_TARGET;
        let channel_type =
            <<gfx_types::ColorFormat as Formatted>::Channel as ChannelTyped>::get_channel_type();

        // Create texture
        let texture = graphic_context.factory.create_texture(
            kind,
            levels,
            bind,
            gfx::memory::Usage::Data,
            Some(channel_type),
        )?;

        // Texture as render target
        let target = graphic_context
            .factory
            .view_texture_as_render_target(&texture, levels, None)?;

        Ok((texture, target))
    }

    fn create_depth(
        graphic_context: &mut GraphicContext,
        size: [u16; 2],
    ) -> Result<
        (
            handle::Texture<Resources, <gfx_types::DepthFormat as Formatted>::Surface>,
            handle::DepthStencilView<Resources, gfx_types::DepthFormat>,
        ),
        gfx::CombinedError,
    > {
        // Texture settings
        let kind = gfx::texture::Kind::D2(size[0], size[1], gfx::texture::AaMode::Single);
        let levels = 1;
        let bind = gfx::memory::Bind::SHADER_RESOURCE | gfx::memory::Bind::DEPTH_STENCIL;
        let channel_type =
            <<gfx_types::DepthFormat as Formatted>::Channel as ChannelTyped>::get_channel_type();

        // Create texture
        let texture = graphic_context.factory.create_texture(
            kind,
            levels,
            bind,
            gfx::memory::Usage::Data,
            Some(channel_type),
        )?;

        // Texture as render target
        let target = graphic_context
            .factory
            .view_texture_as_depth_stencil_trivial(&texture)?;

        Ok((texture, target))
    }

    #[inline]
    pub fn render_target(
        &self,
    ) -> handle::RenderTargetView<gfx_device::Resources, gfx_types::ColorFormat> {
        self.render_target.clone()
    }

    #[inline]
    pub fn start_draw<'a>(
        &'a mut self,
        encoder: &'a mut gfx_types::GraphicsEncoder,
    ) -> CanvasPainter<'a> {
        CanvasPainter {
            encoder,
            canvas: self,
        }
    }
}

// ------- //
// Drawing //
// ------- //

pub struct CanvasPainter<'a> {
    encoder: &'a mut gfx_types::GraphicsEncoder,
    canvas: &'a mut Canvas,
}

impl<'a> CanvasPainter<'a> {
    pub fn draw_mesh(self, mesh: &Mesh, mat: &Material, trans: &Transform) -> Self {
        self
    }
}
