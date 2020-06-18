use crate::graphics::GraphicContext;
use crate::gui::NodeId;
use specs::{Entity, World};

pub struct GuiBuilder<'a> {
    pub world: &'a mut World,
    pub graphics: &'a mut GraphicContext,
}

impl<'a> GuiBuilder<'a> {
    pub fn new(world: &'a mut World, graphics: &'a mut GraphicContext) -> Self {
        GuiBuilder { world, graphics }
    }

    pub fn add<T>(mut self, widget_builder: T) -> Self
    where
        T: WidgetBuilder,
    {
        widget_builder.build(&mut self.world, &mut self.graphics);
        self
    }
}

pub trait WidgetBuilder {
    fn build(self, world: &mut World, graphics: &mut GraphicContext) -> (Entity, NodeId);
}
