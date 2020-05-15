use crate::camera::CameraProjection;
use crate::comp::Transform;
use crate::draw2d::Canvas;
use crate::gfx_types::{self, gizmo_pipe, pipe, DepthTarget, PipelineBundle, RenderTarget};
use crate::graphics::GraphicContext;
use crate::gui::{GuiDrawable, GuiMesh};
use crate::render::{ChannelPair, Material};
use crate::res::{DeviceDimensions, ViewPort};
use gfx_device::{CommandBuffer, Resources};
use glutin::dpi::{LogicalSize, PhysicalSize};
use nalgebra::{Matrix4, Vector3};
use specs::{Join, ReadExpect, ReadStorage, System};

pub struct DrawGuiSystem {
    channel: ChannelPair<Resources, CommandBuffer>,
    canvas: Canvas,
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
    gui_drawables: ReadStorage<'a, GuiDrawable>,
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
            canvas,
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
            gizmo_pipe_bundle,
            view_port,
            device_dim,
            materials,
            transforms,
            gui_meshes,
            gui_drawables,
        } = data;

        let device_physical_size = *device_dim.physical_size();
        let dpi_factor = device_dim.dpi_factor() as f32;
        self.camera.set_device_size((
            device_physical_size.width as u16,
            device_physical_size.height as u16,
        ));

        match self.channel.recv_block() {
            Ok(mut encoder) => {
                // for (drawable,) in (&drawables,).join() {
                //     match drawable {
                //         GuiDrawable::Text(_txt) => draw_txt(),
                //         GuiDrawable::Rectangle(_rect) => { /* Draw to canvas */ }
                //     }
                // }

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
                                // proj: self.camera.orthographic([0.0, 0.0, 0.0]).into(),
                                // proj: glm::Mat4x4::identity().into(),
                                proj: create_matrix(device_physical_size, dpi_factor).into(),
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

fn draw_txt() {}

/// Create the view matrix of the GUI.
///
/// TODO:
///   - Explain missing z coordinate
///   - Explain scale to cancel window stretch
///   - Explain translate by whole window size
fn create_matrix<P>(device_size: P, dpi_factor: f32) -> Matrix4<f32>
where
    P: Into<PhysicalSize>,
{
    let PhysicalSize {
        width: device_w,
        height: device_h,
    } = device_size.into();

    // The normalised device coordinates (-1 to 1) will be mapped to a -500 to +500 pixels.
    //
    // Higher DPI factor means more pixels can fit into the same area.
    let pixel_scale = 1000.0 * dpi_factor;
    let (w, h) = (device_w as f32 / pixel_scale, device_h as f32 / pixel_scale);

    let mut m: Matrix4<f32> = Matrix4::identity();

    // Scale by negating stretch caused by window
    let (sx, sy) = (1.0 / w, 1.0 / h);
    m.prepend_nonuniform_scaling_mut(&Vector3::new(sx, sy, 0.0));

    // Translate so origin is in bottom left of window
    m.prepend_translation_mut(&Vector3::new(-w, h, 0.0));

    m
}
