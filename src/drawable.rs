use crate::graphics::GraphicContext;

pub trait Drawable {
    fn draw(&self, context: &GraphicContext);
}
