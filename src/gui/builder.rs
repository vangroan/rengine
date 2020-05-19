use crate::graphics::GraphicContext;
use crate::gui::{GuiGraph, NodeId};
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

// pub struct WidgetBuilder<'a> {
//     entity: Entity,
//     world: &'a mut World,
//     parent: Option<WidgetId>,
// }

// impl<'a> WidgetBuilder<'a> {
//     pub fn child_of(mut self, parent: WidgetId) -> Self {
//         self.parent = Some(parent);
//         self
//     }

//     pub fn build(self) -> Option<WidgetId> {
//         let WidgetBuilder {
//             entity,
//             world,
//             parent,
//         } = self;

//         // TODO: Return error when GUI is not in world
//         // world
//         //     .res
//         //     .try_fetch_mut::<GuiGraph>()
//         //     .map(|mut gui| gui.insert_entity(entity, parent))
//         unimplemented!()
//     }
// }

pub trait WidgetBuilder {
    fn build(self, world: &mut World, graphics: &mut GraphicContext) -> (Entity, NodeId);
}
