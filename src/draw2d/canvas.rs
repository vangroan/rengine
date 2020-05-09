use crate::gfx_types;
use crate::graphics::GraphicContext;
use gfx;
use gfx::{
    format::{ChannelTyped, Formatted},
    Factory,
};
use gfx_device::Resources;

pub struct Canvas {
    texture: gfx::handle::Texture<Resources, <gfx_types::ColorFormat as Formatted>::Surface>,
    render_target: gfx::handle::RenderTargetView<Resources, gfx_types::ColorFormat>,
}

impl Canvas {
    pub fn new(
        graphic_context: &mut GraphicContext,
        width: u32,
        height: u32,
    ) -> Result<Canvas, gfx::CombinedError> {
        let kind =
            gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
        let levels = 1;
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
        let render_target = graphic_context
            .factory
            .view_texture_as_render_target(&texture, levels, None)?;

        Ok(Canvas {
            texture,
            render_target,
        })
    }
}
