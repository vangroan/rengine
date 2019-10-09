use crate::text::TextBatch;
use specs::{Component, DenseVecStorage};

/// Two dimensional shapes for rendering interfaces.
#[derive(Component)]
#[storage(DenseVecStorage)]
pub enum GuiDrawable {
    Rectangle(GuiRectangle),

    Text(TextBatch),
}

pub struct GuiRectangle {}
