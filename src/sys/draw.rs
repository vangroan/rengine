use crate::camera::{ActiveCamera, CameraProjection, CameraView};
use crate::comp::{GlTexture, Mesh, Transform};
use crate::gfx_types::{
    self, gizmo_pipe, gloss_pipe, pipe, DepthTarget, PipelineBundle, RenderTarget,
};
use crate::metrics::{builtin_metrics::*, MetricAggregate, MetricHub};
use crate::option::lift2;
use crate::render::{ChannelPair, Gizmo, Lights, Material, PointLight};
use crate::res::ViewPort;

use nalgebra::{Matrix4, Vector4};
use specs::{Join, Read, ReadExpect, ReadStorage, System};

pub struct DrawSystem {
    channel: ChannelPair<gfx_device::Resources, gfx_device::CommandBuffer>,
    pub(crate) render_target: RenderTarget<gfx_device::Resources>,
    pub(crate) depth_target: DepthTarget<gfx_device::Resources>,
}

#[derive(SystemData)]
pub struct DrawSystemData<'a> {
    // metrics: Read<'a, MetricHub>,
    basic_pipe_bundle: ReadExpect<'a, PipelineBundle<pipe::Meta>>,
    gloss_pipe_bundle: ReadExpect<'a, PipelineBundle<gloss_pipe::Meta>>,
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
    lights: ReadExpect<'a, Lights>,
    point_lights: ReadStorage<'a, PointLight>,
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
            // metrics,
            basic_pipe_bundle,
            gloss_pipe_bundle,
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
            lights,
            point_lights,
        } = data;
        match self.channel.recv_block() {
            Ok(mut encoder) => {
                // let mut render_timer = metrics.timer(GRAPHICS_RENDER, MetricAggregate::Maximum);
                // let mut _draw_call_counter =
                //     metrics.counter(GRAPHICS_DRAW_CALLS, MetricAggregate::Sum);

                // Without a camera, we draw according to the default OpenGL behaviour
                let (proj_matrix, view_matrix, eye) = active_camera
                    .camera_entity()
                    .and_then(|entity| lift2(cam_projs.get(entity), cam_views.get(entity)))
                    .map(|(proj, view)| {
                        // let pos = view.position();
                        // TODO: Allow user to select between orthographic and perspective at runtime
                        (
                            proj.perspective(),
                            view.view_matrix(),
                            view.position().to_homogeneous(),
                        )
                    })
                    .unwrap_or((
                        Matrix4::identity(),
                        Matrix4::identity(),
                        Vector4::new(0.0, 0.0, 0.0, 1.0),
                    ));

                // Send lights to graphics card
                let max_lights = lights.max_num();
                let mut light_count = 0;
                for (offset, (light_trans, point_light)) in (&transforms, &point_lights)
                    .join()
                    .enumerate()
                    .take(max_lights)
                {
                    let pos = light_trans.position();
                    let light_params = gfx_types::LightParams {
                        pos: [pos.x, pos.y, pos.z, 1.0],
                        ambient: point_light.ambient,
                        diffuse: point_light.diffuse,
                        specular: point_light.specular,
                    };

                    // Send light to graphics card
                    encoder
                        .update_buffer(&lights.buffer(), &[light_params], offset)
                        .expect("Failed to update buffer");

                    light_count += 1;
                }

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
                        Material::Gloss { texture, material } => {
                            // Send material to graphics card
                            encoder
                                .update_buffer(
                                    &material.material_buf,
                                    &[material.clone().into()],
                                    0,
                                )
                                .expect("Failed to update buffer");

                            // Surface Normal Matrix
                            let model_matrix = trans.matrix();
                            let mut normal_matrix = model_matrix;
                            normal_matrix.try_inverse_mut();
                            normal_matrix.transpose_mut();

                            // Prepare data
                            let data = gloss_pipe::Data {
                                vbuf: mesh.vbuf.clone(),
                                sampler: (
                                    texture.bundle.view.clone(),
                                    texture.bundle.sampler.clone(),
                                ),
                                material: material.material_buf.clone(),
                                lights: lights.buffer().clone(),
                                num_lights: light_count,
                                eye: eye.into(),
                                normal_matrix: normal_matrix.into(),
                                model: model_matrix.into(),
                                view: view_matrix.into(),
                                proj: proj_matrix.into(),
                                // The rectangle to allow rendering within
                                scissor: view_port.rect,
                                render_target: self.render_target.clone(),
                                depth_target: self.depth_target.clone(),
                            };

                            encoder.draw(&mesh.slice, &gloss_pipe_bundle.pso, &data);
                        }
                        _ => unimplemented!(),
                    }
                }

                // Second pass for drawing debug gizmos
                for (ref mesh, ref _mat, ref trans, ref _gizmo) in
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

                // render_timer.stop();
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}
