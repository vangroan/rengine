use super::{create_gui_proj_matrix, GuiMesh};
use crate::camera::CameraProjection;
use crate::comp::Transform;
use crate::draw2d::Canvas;
use crate::gfx_types::{self, gizmo_pipe, pipe, DepthTarget, PipelineBundle, RenderTarget};
use crate::render::{ChannelPair, Material};
use crate::res::{DeviceDimensions, ViewPort};
use gfx_device::{CommandBuffer, Resources};
use specs::{Join, ReadExpect, ReadStorage, System};

pub struct DrawGuiSystem {
    channel: ChannelPair<Resources, CommandBuffer>,
    _canvas: Canvas,
    pub(crate) render_target: RenderTarget<gfx_device::Resources>,
    pub(crate) depth_target: DepthTarget<gfx_device::Resources>,
    camera: CameraProjection,
}

#[derive(SystemData)]
pub struct DrawGuiSystemData<'a> {
    basic_pipe_bundle: ReadExpect<'a, PipelineBundle<pipe::Meta>>,
    gizmo_pipe_bundle: ReadExpect<'a, PipelineBundle<gizmo_pipe::Meta>>,
    view_port: ReadExpect<'a, ViewPort>,
    device_dim: ReadExpect<'a, DeviceDimensions>,
    materials: ReadStorage<'a, Material>,
    transforms: ReadStorage<'a, Transform>,
    gui_meshes: ReadStorage<'a, GuiMesh>,
}

impl DrawGuiSystem {
    pub fn new(
        channel: ChannelPair<Resources, CommandBuffer>,
        canvas: Canvas,
        render_target: RenderTarget<gfx_device::Resources>,
        depth_target: DepthTarget<gfx_device::Resources>,
    ) -> Self {
        DrawGuiSystem {
            channel,
            _canvas: canvas,
            render_target,
            depth_target,
            camera: CameraProjection::default(),
        }
    }
}

impl<'a> System<'a> for DrawGuiSystem {
    type SystemData = DrawGuiSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let DrawGuiSystemData {
            basic_pipe_bundle,
            view_port,
            device_dim,
            materials,
            transforms,
            gui_meshes,
            ..
        } = data;

        let device_physical_size = *device_dim.physical_size();
        let dpi_factor = device_dim.dpi_factor() as f32;
        self.camera.set_device_size((
            device_physical_size.width as u16,
            device_physical_size.height as u16,
        ));

        let proj_matrix = create_gui_proj_matrix(device_physical_size, dpi_factor);

        match self.channel.recv_block() {
            Ok(mut encoder) => {
                // Draw to screen
                for (ref mesh, ref mat, ref trans) in (&gui_meshes, &materials, &transforms).join()
                {
                    match mat {
                        Material::Basic { texture } => {
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
                                sampler: (
                                    texture.bundle.view.clone(),
                                    texture.bundle.sampler.clone(),
                                ),
                                transforms: mesh.transbuf.clone(),
                                view: glm::Mat4x4::identity().into(),
                                proj: proj_matrix.into(),
                                // The rectangle to allow rendering within
                                scissor: view_port.rect,
                                render_target: self.render_target.clone(),
                                depth_target: self.depth_target.clone(),
                            };

                            encoder.draw(&mesh.slice, &basic_pipe_bundle.pso, &data);
                        }
                        _ => unimplemented!(),
                    }
                }

                self.channel
                    .send_block(encoder)
                    .expect("GUI render failed sending encoder back to main loop");
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}
