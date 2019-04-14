use crate::comp::{Mesh, Transform};
use crate::gfx_types::{self, pipe, PipelineStateObject, RenderTarget};
use crate::graphics::ChannelPair;
use crate::res::ViewPort;
use specs::{Join, ReadExpect, ReadStorage, System};
use std::error::Error;

pub struct DrawSystem {
    channel: ChannelPair<gfx_device::Resources, gfx_device::CommandBuffer>,
    pub(crate) render_target: RenderTarget<gfx_device::Resources>,
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
        ReadExpect<'a, ViewPort>,
        ReadStorage<'a, Mesh>,
        ReadStorage<'a, Transform>,
    );

    fn run(&mut self, (pso, view_port, meshes, transforms): Self::SystemData) {
        match self.channel.recv_block() {
            Ok(mut encoder) => {
                for (ref mesh, ref trans) in (&meshes, &transforms).join() {
                    // Convert to pipeline transform type
                    let trans = gfx_types::Transform {
                        transform: trans.matrix().into(),
                    };

                    // Send transform to graphics card
                    encoder
                        .update_buffer(&mesh.transbuf, &[trans], 0)
                        .expect("Failed to update buffer");

                    let data = pipe::Data {
                        vbuf: mesh.vbuf.clone(),
                        transforms: mesh.transbuf.clone(),
                        // TODO: Camera position and zoom
                        // view: view_port.scale.into(),
                        // The rectangle to allow rendering within
                        scissor: view_port.rect,
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
