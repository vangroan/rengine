use crate::camera::{ActiveCamera, CameraProjection, CameraView};
use crate::comp::{GlTexture, Mesh, Transform};
use crate::gfx_types::{self, gizmo_pipe, pipe, DepthTarget, PipelineBundle, RenderTarget};
use crate::option::lift2;
use crate::render::{ChannelPair, Gizmo, Material};
use crate::res::ViewPort;
use nalgebra::Matrix4;
use specs::{Join, Read, ReadExpect, ReadStorage, System, SystemData};

pub struct DrawSystem {
    channel: ChannelPair<gfx_device::Resources, gfx_device::CommandBuffer>,
    pub(crate) render_target: RenderTarget<gfx_device::Resources>,
    pub(crate) depth_target: DepthTarget<gfx_device::Resources>,
}

#[derive(SystemData)]
pub struct DrawSystemData<'a> {
    basic_pipe_bundle: ReadExpect<'a, PipelineBundle<pipe::Meta>>,
    gizmo_pipe_bundle: ReadExpect<'a, PipelineBundle<gizmo_pipe::Meta>>,
    view_port: ReadExpect<'a, ViewPort>,
    active_camera: Read<'a, ActiveCamera>,
    meshes: ReadStorage<'a, Mesh>,
    materials: ReadStorage<'a, Material>,
    textures: ReadStorage<'a, GlTexture>,
    transforms: ReadStorage<'a, Transform>,
    cam_views: ReadStorage<'a, CameraView>,
    cam_projs: ReadStorage<'a, CameraProjection>,
    gizmos: ReadStorage<'a, Gizmo>,
}

impl DrawSystem {
    pub fn new(
        channel: ChannelPair<gfx_device::Resources, gfx_device::CommandBuffer>,
        render_target: RenderTarget<gfx_device::Resources>,
        depth_target: DepthTarget<gfx_device::Resources>,
    ) -> Self {
        DrawSystem {
            channel,
            render_target,
            depth_target,
        }
    }
}

impl DrawSystem {
    fn draw_gizmo(
        &self,
        encoder: &mut gfx::Encoder<gfx_device::Resources, gfx_device::CommandBuffer>,
        gizmo_pipe_bundle: &gfx_types::PipelineBundle<gizmo_pipe::Meta>,
        mesh: &Mesh,
        transform: &Transform,
        view_matrix: Matrix4<f32>,
        proj_matrix: Matrix4<f32>,
        view_port: &ViewPort,
    ) {
        let data = gizmo_pipe::Data {
            vbuf: mesh.vbuf.clone(),
            model: transform.matrix().into(),
            view: view_matrix.into(),
            proj: proj_matrix.into(),
            // The rectangle to allow rendering within
            scissor: view_port.rect,
            render_target: self.render_target.clone(),
            depth_target: self.depth_target.clone(),
        };

        encoder.draw(&mesh.slice, &gizmo_pipe_bundle.pso, &data);
    }
}

impl<'a> System<'a> for DrawSystem {
    type SystemData = DrawSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let DrawSystemData {
            basic_pipe_bundle,
            gizmo_pipe_bundle,
            view_port,
            active_camera,
            meshes,
            materials,
            textures,
            transforms,
            cam_views,
            cam_projs,
            gizmos,
        } = data;
        match self.channel.recv_block() {
            Ok(mut encoder) => {
                // Without a camera, we draw according to the default OpenGL behaviour
                let (proj_matrix, view_matrix) = active_camera
                    .camera_entity()
                    .and_then(|entity| lift2(cam_projs.get(entity), cam_views.get(entity)))
                    .map(|(proj, view)| {
                        // let pos = view.position();
                        // TODO: Allow user to select between orthographic and perspective at runtime
                        (proj.perspective(), view.view_matrix())
                    })
                    .unwrap_or((Matrix4::identity(), Matrix4::identity()));

                // for (ref mesh, ref tex, ref trans) in (&meshes, &textures, &transforms).join() {
                //     // Convert to pipeline transform type
                //     let trans = gfx_types::Transform {
                //         transform: trans.matrix().into(),
                //     };

                //     // Send transform to graphics card
                //     encoder
                //         .update_buffer(&mesh.transbuf, &[trans], 0)
                //         .expect("Failed to update buffer");

                //     // Prepare data
                //     let data = pipe::Data {
                //         vbuf: mesh.vbuf.clone(),
                //         sampler: (tex.bundle.view.clone(), tex.bundle.sampler.clone()),
                //         transforms: mesh.transbuf.clone(),
                //         view: view_matrix.into(),
                //         proj: proj_matrix.into(),
                //         // The rectangle to allow rendering within
                //         scissor: view_port.rect,
                //         render_target: self.render_target.clone(),
                //         depth_target: self.depth_target.clone(),
                //     };

                //     encoder.draw(&mesh.slice, &basic_pipe_bundle.pso, &data);
                // }

                for (ref mesh, ref mat, ref trans) in (&meshes, &materials, &transforms).join() {
                    // Choose pipeline based on material
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
                                view: view_matrix.into(),
                                proj: proj_matrix.into(),
                                // The rectangle to allow rendering within
                                scissor: view_port.rect,
                                render_target: self.render_target.clone(),
                                depth_target: self.depth_target.clone(),
                            };

                            encoder.draw(&mesh.slice, &basic_pipe_bundle.pso, &data);
                        }
                        Material::Gizmo => {
                            self.draw_gizmo(
                                &mut encoder,
                                &*gizmo_pipe_bundle,
                                mesh,
                                trans,
                                view_matrix,
                                proj_matrix,
                                &*view_port,
                            );
                            // Prepare data
                            // let data = gizmo_pipe::Data {
                            //     vbuf: mesh.vbuf.clone(),
                            //     model: trans.matrix().into(),
                            //     view: view_matrix.into(),
                            //     proj: proj_matrix.into(),
                            //     // The rectangle to allow rendering within
                            //     scissor: view_port.rect,
                            //     render_target: self.render_target.clone(),
                            //     depth_target: self.depth_target.clone(),
                            // };

                            // encoder.draw(&mesh.slice, &gizmo_pipe_bundle.pso, &data);
                        }
                        _ => unimplemented!(),
                    }
                }

                // Second pass for drawing debug gizmos
                for (ref mesh, ref mat, ref trans, ref _gizmo) in
                    (&meshes, &materials, &transforms, &gizmos).join()
                {
                    self.draw_gizmo(
                        &mut encoder,
                        &*gizmo_pipe_bundle,
                        mesh,
                        trans,
                        view_matrix,
                        proj_matrix,
                        &*view_port,
                    );
                }

                if let Err(err) = self.channel.send_block(encoder) {
                    eprintln!("{}", err);
                }
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}
