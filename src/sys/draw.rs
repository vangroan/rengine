use crate::comp::Mesh;
use crate::drawable::Drawable;
use crate::gfx_types::{PipelineStateObject, RenderTarget};
use crate::graphics::ChannelPair;
use specs::{Join, ReadExpect, ReadStorage, System};
use std::error::Error;

pub struct DrawSystem {
    channel: ChannelPair<gfx_device::Resources, gfx_device::CommandBuffer>,
    render_target: RenderTarget<gfx_device::Resources>,
}

impl DrawSystem {
    pub fn new(
        channel: ChannelPair<gfx_device::Resources, gfx_device::CommandBuffer>,
        render_target: RenderTarget<gfx_device::Resources>,
    ) -> Self {
        DrawSystem {
            channel,
            render_target,
        }
    }
}

impl<'a> System<'a> for DrawSystem {
    type SystemData = (ReadExpect<'a, PipelineStateObject>, ReadStorage<'a, Mesh>);

    fn run(&mut self, (pso, meshes): Self::SystemData) {
        match self.channel.recv_block() {
            Ok(mut encoder) => {
                for (ref mesh,) in (&meshes,).join() {
                    mesh.draw(&mut encoder, &pso, &self.render_target);
                }

                if let Err(err) = self.channel.send_block(encoder) {
                    eprintln!("{}, {}", err, err.description());
                }
            }
            Err(err) => eprintln!("{}, {}", err, err.description()),
        }
    }
}
