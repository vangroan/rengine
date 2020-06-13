use crate::colors::{self, Color};
use gfx_glyph::{FontId, Layout, Section, Text};
use specs::{Component, DenseVecStorage};

#[derive(Component, Default)]
#[storage(DenseVecStorage)]
pub struct TextBatch {
    fragments: Vec<TextFragment>,
    layout: LayoutSettings,
    pub z: f32,
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

    #[inline]
    pub fn add_fragment(&mut self, fragment: TextFragment) {
        self.fragments.push(fragment);
    }

    #[inline]
    pub fn set_align(&mut self, align_v: TextAlignVertical, align_h: TextAlignHorizontal) {
        self.layout.align_v = align_v;
        self.layout.align_h = align_h;
    }

    #[inline]
    pub fn set_z_depth(&mut self, z_depth: f32) {
        self.z = z_depth;
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

    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    pub fn with_align(mut self, align_v: TextAlignVertical, align_h: TextAlignHorizontal) -> Self {
        self.set_align(align_v, align_h);
        self
    }

    pub fn as_section(&self, dpi_factor: f32, bounds: [f32; 2]) -> Section {
        let texts: Vec<_> = self
            .fragments
            .iter()
            .map(|fragment| {
                Text::new(&fragment.content)
                    .with_color(fragment.color)
                    .with_scale(fragment.scale * dpi_factor)
                    .with_font_id(fragment.font_id)
                    .with_z(self.z)
            })
            .collect();

        let mut section = Section::default();
        for text in texts {
            section = section.add_text(text);
        }
        section.bounds = (bounds[0], bounds[1]);
        section.layout = Layout::default_wrap()
            .h_align(match self.layout.align_h {
                TextAlignHorizontal::Left => gfx_glyph::HorizontalAlign::Left,
                TextAlignHorizontal::Center => gfx_glyph::HorizontalAlign::Center,
                TextAlignHorizontal::Right => gfx_glyph::HorizontalAlign::Right,
            })
            .v_align(match self.layout.align_v {
                TextAlignVertical::Top => gfx_glyph::VerticalAlign::Top,
                TextAlignVertical::Center => gfx_glyph::VerticalAlign::Center,
                TextAlignVertical::Bottom => gfx_glyph::VerticalAlign::Bottom,
            });

        section
    }
}

pub struct LayoutSettings {
    pub align_v: TextAlignVertical,
    pub align_h: TextAlignHorizontal,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        LayoutSettings {
            align_v: TextAlignVertical::Center,
            align_h: TextAlignHorizontal::Center,
        }
    }
}

pub enum TextAlignVertical {
    Top,
    Center,
    Bottom,
}

pub enum TextAlignHorizontal {
    Left,
    Center,
    Right,
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
