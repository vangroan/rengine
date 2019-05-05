use crate::gfx_types::RenderTarget;
use crate::gfx_types::{PipelineBundle, PipelineStateObject, ShaderProgram};
use crate::render::channel::ChannelPair;
use crate::render::DrawFactory;

pub struct GizmoDrawSystem {
    channel: ChannelPair<gfx_device::Resources, gfx_device::CommandBuffer>,
    pub(crate) render_target: RenderTarget<gfx_device::Resources>,
}

impl GizmoDrawSystem {
    pub fn new(
        channel: ChannelPair<gfx_device::Resources, gfx_device::CommandBuffer>,
        render_target: RenderTarget<gfx_device::Resources>,
    ) -> Self {
        GizmoDrawSystem {
            channel,
            render_target,
        }
    }
}

impl DrawFactory for GizmoDrawSystem {
    fn create() -> Self {
        unimplemented!()
    }
}

/// New type for Gizmo specific pipeline
pub struct GizmoPipelineBundle(PipelineBundle);

impl GizmoPipelineBundle {
    pub fn new(pso: PipelineStateObject, program: ShaderProgram) -> Self {
        GizmoPipelineBundle(PipelineBundle { pso, program })
    }

    #[inline]
    pub fn bundle(&self) -> &PipelineBundle {
        &self.0
    }
}
