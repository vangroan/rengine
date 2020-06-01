use crate::graphics::GraphicContext;
use crate::render::ChannelPair;
use crate::text::TextBatch;
use gfx_device::{CommandBuffer, Resources};
use gfx_glyph::Section;
use specs::{Join, ReadStorage, World};

pub struct DrawTextSystem {
    channel: ChannelPair<Resources, CommandBuffer>,
}

impl DrawTextSystem {
    pub fn new(channel: ChannelPair<Resources, CommandBuffer>) -> Self {
        DrawTextSystem { channel }
    }
}

impl DrawTextSystem {
    pub fn render(&mut self, world: &mut World, graphics: &mut GraphicContext) {
        match self.channel.recv_block() {
            Ok(mut encoder) => {
                let (text_batches,): (ReadStorage<'_, TextBatch>,) = world.system_data();

                // Project text batches to a form that GlyphBrush can use
                let varied_sections: Vec<Section> = text_batches
                    .join()
                    .map(|text_batch| text_batch.as_section())
                    .collect();

                for varied_section in varied_sections.into_iter() {
                    graphics.glyph_brush.queue(varied_section);
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
