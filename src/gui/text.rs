use super::layout::GlobalPosition;
use crate::colors::Color;
use crate::graphics::GraphicContext;
use crate::render::ChannelPair;
use gfx_device::{CommandBuffer, Resources};
use gfx_glyph::{FontId, SectionText, VariedSection};
use specs::{Component, DenseVecStorage};
use specs::{Join, ReadStorage, World};

// ------- //
// Systems //
// ------- //

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
                let DrawTextSystemData {
                    text_batches,
                    global_positions,
                } = world.system_data();

                // Project text batches to a form that GlyphBrush can use
                let varied_sections: Vec<VariedSection> = (&text_batches, &global_positions)
                    .join()
                    .map(|(text_batch, pos)| {
                        let mut section = text_batch.as_section();
                        section.screen_position = pos.into();
                        section
                    })
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

#[derive(SystemData)]
pub struct DrawTextSystemData<'a> {
    text_batches: ReadStorage<'a, TextBatch>,
    global_positions: ReadStorage<'a, GlobalPosition>,
}

// ---------- //
// Components //
// ---------- //

#[derive(Component, Default)]
#[storage(DenseVecStorage)]
pub struct TextBatch {
    fragments: Vec<TextFragment>,
}

impl TextBatch {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add<C>(&mut self, text: &str, color: C)
    where
        C: Into<Color>,
    {
        self.fragments.push(TextFragment {
            content: text.to_owned(),
            color: color.into(),
            _font: FontId::default(),
        });
    }

    /// Clears all existing text fragments and replaces
    /// them with the given text string.
    pub fn replace<C>(&mut self, text: &str, color: C)
    where
        C: Into<Color>,
    {
        self.fragments.clear();
        self.add(text, color);
    }

    pub fn with<C>(mut self, text: &str, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.add(text, color);

        self
    }

    pub fn as_section(&self) -> VariedSection {
        let sections: Vec<SectionText> = self
            .fragments
            .iter()
            .map(|fragment| SectionText {
                text: &fragment.content,
                color: fragment.color,
                ..SectionText::default()
            })
            .collect();

        VariedSection {
            text: sections,
            ..VariedSection::default()
        }
    }
}

pub struct TextFragment {
    /// Owned textual string content
    content: String,

    /// Text color to be rendered
    color: Color,

    /// Handle to font stored in glyph brush
    _font: FontId,
}
