use glutin::dpi::LogicalSize;
use specs::{Component, DenseVecStorage};

/// Axis-aligned bounding box in logical pixel size.
#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct WidgetBounds {
    width: f32,
    height: f32,
}

impl WidgetBounds {
    pub fn new(width: f32, height: f32) -> Self {
        WidgetBounds { width, height }
    }

    pub fn from_size(size: LogicalSize) -> Self {
        WidgetBounds {
            width: size.width as f32,
            height: size.height as f32,
        }
    }
}
