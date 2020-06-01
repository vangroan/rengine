use super::super::layout;
use super::TextBatch;
use crate::gfx_types::{DepthTarget, RenderTarget};
use crate::graphics::GraphicContext;
use crate::render::ChannelPair;
use crate::res::DeviceDimensions;
use gfx_device::{CommandBuffer, Resources};
use gfx_glyph::Section;
use specs::{Join, ReadExpect, ReadStorage, System, World};

pub struct DrawTextSystem {
    channel: ChannelPair<Resources, CommandBuffer>,
    pub(crate) render_target: RenderTarget<gfx_device::Resources>,
    pub(crate) depth_target: DepthTarget<gfx_device::Resources>,
}

#[derive(SystemData)]
pub struct DrawTextSystemData<'a> {
    device_dim: ReadExpect<'a, DeviceDimensions>,
    global_positions: ReadStorage<'a, layout::GlobalPosition>,
    text_batches: ReadStorage<'a, TextBatch>,
}

impl DrawTextSystem {
    pub fn new(
        channel: ChannelPair<Resources, CommandBuffer>,
        render_target: RenderTarget<gfx_device::Resources>,
        depth_target: DepthTarget<gfx_device::Resources>,
    ) -> Self {
        DrawTextSystem {
            channel,
            render_target,
            depth_target,
        }
    }
}

impl<'a> System<'a> for DrawTextSystem {
    type SystemData = DrawTextSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {}
}

impl DrawTextSystem {
    pub fn render(&mut self, world: &mut World, graphics: &mut GraphicContext) {
        let dpi_factor = world.read_resource::<DeviceDimensions>().dpi_factor() as f32;
        match self.channel.recv_block() {
            Ok(mut encoder) => {
                let (text_batches,): (ReadStorage<'_, TextBatch>,) = world.system_data();

                // Project text batches to a form that GlyphBrush can use
                let sections: Vec<Section> = text_batches
                    .join()
                    .map(|text_batch| text_batch.as_section(dpi_factor))
                    .collect();

                for section in sections.into_iter() {
                    graphics.glyph_brush.queue(section);
                }

                graphics
                    .glyph_brush
                    .use_queue()
                    .draw(&mut encoder, &graphics.render_target)
                    .expect("Failed drawing text queue");

                self.channel
                    .send_block(encoder)
                    .expect("Text render failed sending encoder back to main loop");
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}
