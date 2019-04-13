use crate::gfx_types::{GraphicsEncoder, PipelineStateObject, RenderTarget};

pub trait Drawable {
    fn draw(
        &self,
        encoder: &mut GraphicsEncoder,
        pso: &PipelineStateObject,
        target: &RenderTarget<gfx_device::Resources>,
    );
}
