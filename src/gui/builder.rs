use specs::{Entity, World};

use crate::gui::{GuiGraph, WidgetId};

pub trait GuiBuilder {
    /// Initialize a builder using
    /// the given entity.
    fn create_widget(&mut self, entity: Entity) -> WidgetBuilder;
}

impl GuiBuilder for World {
    fn create_widget(&mut self, entity: Entity) -> WidgetBuilder {
        WidgetBuilder {
            entity,
            world: self,
            parent: None,
        }
    }
}

pub struct WidgetBuilder<'a> {
    entity: Entity,
    world: &'a mut World,
    parent: Option<WidgetId>,
}

impl<'a> WidgetBuilder<'a> {
    pub fn child_of(mut self, parent: WidgetId) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn build(self) -> Option<WidgetId> {
        let WidgetBuilder {
            entity,
            world,
            parent,
        } = self;

        // TODO: Return error when GUI is not in world
        // world
        //     .res
        //     .try_fetch_mut::<GuiGraph>()
        //     .map(|mut gui| gui.insert_entity(entity, parent))
        unimplemented!()
    }
}
