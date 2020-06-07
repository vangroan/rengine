use super::super::layout;
use super::TextBatch;
use crate::gfx_types::{DepthTarget, RenderTarget};
use crate::render::ChannelPair;
use crate::res::DeviceDimensions;
use gfx_device::{CommandBuffer, Resources};
use gfx_glyph::{GlyphBrush, Section};
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

        match self.channel.recv_block() {
            Ok(mut encoder) => {
                // Project text batches to a form that GlyphBrush can use
                let sections: Vec<Section> = (&text_batches, &global_positions, &bounds_rects)
                    .join()
                    .map(|(text_batch, pos, bounds)| {
                        let mut section = text_batch.as_section(dpi_factor, (*bounds).into());
                        section.screen_position = pos.into();
                        section
                    })
                    .collect();

                for section in sections.into_iter() {
                    self.glyph_brush.queue(section);
                }

                self.glyph_brush
                    .use_queue()
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
