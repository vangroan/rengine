use crate::colors::{self, Color};
use gfx_glyph::{FontId, Section, Text};
use specs::{Component, DenseVecStorage};

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
            ..TextFragment::default()
        });
    }

    pub fn add_fragment(&mut self, fragment: TextFragment) {
        self.fragments.push(fragment);
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

    pub fn as_section(&self, dpi_factor: f32) -> Section {
        let text: Vec<_> = self
            .fragments
            .iter()
            .map(|fragment| {
                Text::new(&fragment.content)
                    .with_color(fragment.color)
                    .with_scale(fragment.scale * dpi_factor)
                    .with_font_id(fragment.font_id)
                    .with_z(1.0)
            })
            .collect();

        Section::default().with_text(text)
    }
}

pub struct TextFragment {
    /// Owned textual string content
    content: String,

    /// Text color to be rendered
    color: Color,

    /// Text logical size
    scale: f32,

    /// Handle to font stored in glyph brush
    font_id: FontId,
}

impl Default for TextFragment {
    fn default() -> Self {
        TextFragment {
            content: "".to_owned(),
            color: colors::WHITE,
            scale: 16.0,
            font_id: FontId::default(),
        }
    }
}
