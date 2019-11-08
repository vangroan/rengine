use crate::colors::Color;
use gfx_glyph::{FontId, SectionText, VariedSection};
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
