use super::super::layout;
use super::TextBatch;
use crate::gfx_types::{DepthTarget, RenderTarget};
use crate::render::ChannelPair;
use crate::res::DeviceDimensions;
use gfx_device::{CommandBuffer, Resources};
use gfx_glyph::{GlyphBrush, Section};
use glutin::dpi::PhysicalSize;
use specs::{Join, ReadExpect, ReadStorage, System};

pub struct DrawTextSystem {
    channel: ChannelPair<Resources, CommandBuffer>,
    pub(crate) render_target: RenderTarget<gfx_device::Resources>,
    pub(crate) depth_target: DepthTarget<gfx_device::Resources>,
    glyph_brush: GlyphBrush<gfx_device::Resources, gfx_device::Factory>,
}

#[derive(SystemData)]
pub struct DrawTextSystemData<'a> {
    device_dim: ReadExpect<'a, DeviceDimensions>,
    global_positions: ReadStorage<'a, layout::GlobalPosition>,
    bounds_rects: ReadStorage<'a, layout::BoundsRect>,
    text_batches: ReadStorage<'a, TextBatch>,
}

impl DrawTextSystem {
    pub fn new(
        channel: ChannelPair<Resources, CommandBuffer>,
        render_target: RenderTarget<gfx_device::Resources>,
        depth_target: DepthTarget<gfx_device::Resources>,
        glyph_brush: GlyphBrush<gfx_device::Resources, gfx_device::Factory>,
    ) -> Self {
        DrawTextSystem {
            channel,
            render_target,
            depth_target,
            glyph_brush,
        }
    }
}

impl<'a> System<'a> for DrawTextSystem {
    type SystemData = DrawTextSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let DrawTextSystemData {
            device_dim,
            global_positions,
            bounds_rects,
            text_batches,
        } = data;

        let dpi_factor = device_dim.dpi_factor() as f32;
        // z-axis is for depth and sorting
        let nearz = -65535.;
        let farz = 65535.;
        let transform = create_text_matrix(*device_dim.physical_size(), nearz, farz);

        match self.channel.recv_block() {
            Ok(mut encoder) => {
                // Project text batches to a form that GlyphBrush can use
                let sections: Vec<Section> = (&text_batches, &global_positions, &bounds_rects)
                    .join()
                    .map(|(text_batch, pos, bounds)| {
                        let mut section = text_batch.as_section(dpi_factor, (*bounds).into());
                        // TODO: Change to physical pixel position
                        let new_pos = pos.point() * dpi_factor;
                        section.screen_position = (new_pos.x, new_pos.y);
                        section
                    })
                    .collect();

                for section in sections.into_iter() {
                    self.glyph_brush.queue(section);
                }

                self.glyph_brush
                    .use_queue()
                    .depth_target(&self.depth_target)
                    .transform(transform)
                    .draw(&mut encoder, &self.render_target)
                    .expect("Failed drawing text queue");

                self.channel
                    .send_block(encoder)
                    .expect("Text render failed sending encoder back to main loop");
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}

pub fn create_text_matrix<S>(device_size: S, nearz: f32, farz: f32) -> [[f32; 4]; 4]
where
    S: Into<PhysicalSize>,
{
    let s = device_size.into();
    let (w, h) = (s.width as f32, s.height as f32);
    [
        [2. / w, 0.0, 0.0, 0.0],
        [0.0, 2. / h, 0.0, 0.0],
        [0.0, 0.0, 2. / (farz - nearz), 0.0],
        [-1.0, -1.0, 0.0, 1.0],
    ]
}
