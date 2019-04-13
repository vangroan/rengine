use crate::comp::{Mesh, Transform};
use crate::drawable::Drawable;
use crate::gfx_types::{pipe, PipelineStateObject, RenderTarget};
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
    type SystemData = (
        ReadExpect<'a, PipelineStateObject>,
        ReadStorage<'a, Mesh>,
        ReadStorage<'a, Transform>,
    );

    fn run(&mut self, (pso, meshes, transforms): Self::SystemData) {
        match self.channel.recv_block() {
            Ok(mut encoder) => {
                for (ref mesh, ref trans) in (&meshes, &transforms).join() {
                    let data = pipe::Data {
                        vbuf: mesh.vbuf.clone(),
                        model: trans.matrix().into(),
                        out: self.render_target.clone(),
                    };

                    encoder.draw(&mesh.slice, &pso, &data);
                }

                if let Err(err) = self.channel.send_block(encoder) {
                    eprintln!("{}, {}", err, err.description());
                }
            }
            Err(err) => eprintln!("{}, {}", err, err.description()),
        }
    }
}
