use crate::comp::{Camera, GlTexture, Mesh, Transform};
use crate::gfx_types::{self, pipe, PipelineBundle, RenderTarget};
use crate::graphics::ChannelPair;
use crate::option::lift2;
use crate::res::{ActiveCamera, ViewPort};
use nalgebra::Matrix4;
use specs::{Join, Read, ReadExpect, ReadStorage, System};
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
        ReadExpect<'a, PipelineBundle>,
        ReadExpect<'a, ViewPort>,
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Mesh>,
        ReadStorage<'a, GlTexture>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Camera>,
    );

    fn run(
        &mut self,
        (pipeline, view_port, active_camera, meshes, textures, transforms, cameras): Self::SystemData,
    ) {
        match self.channel.recv_block() {
            Ok(mut encoder) => {
                // Without a camera, we draw according to the default OpenGL behaviour
                let (proj_matrix, view_matrix) = active_camera
                    .camera_entity()
                    .and_then(|entity| lift2(cameras.get(entity), transforms.get(entity)))
                    .map(|(camera, transform)| {
                        let pos = transform.pos;

                        (camera.proj_matrix(pos), camera.view_matrix(pos))
                    })
                    .unwrap_or((Matrix4::identity(), Matrix4::identity()));

                for (ref mesh, ref tex, ref trans) in (&meshes, &textures, &transforms).join() {
                    // Convert to pipeline transform type
                    let trans = gfx_types::Transform {
                        transform: trans.matrix().into(),
                    };

                    // Send transform to graphics card
                    encoder
                        .update_buffer(&mesh.transbuf, &[trans], 0)
                        .expect("Failed to update buffer");

                    // Prepare data
                    let data = pipe::Data {
                        vbuf: mesh.vbuf.clone(),
                        sampler: (tex.bundle.view.clone(), tex.bundle.sampler.clone()),
                        transforms: mesh.transbuf.clone(),
                        // TODO: Camera position and zoom
                        view: view_matrix.into(),
                        proj: proj_matrix.into(),
                        // The rectangle to allow rendering within
                        scissor: view_port.rect,
                        out: self.render_target.clone(),
                    };

                    encoder.draw(&mesh.slice, &pipeline.pso, &data);
                }

                if let Err(err) = self.channel.send_block(encoder) {
                    eprintln!("{}, {}", err, err.description());
                }
            }
            Err(err) => eprintln!("{}, {}", err, err.description()),
        }
    }
}
