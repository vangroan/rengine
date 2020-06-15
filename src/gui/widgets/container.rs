use super::super::{
    layout, next_widget_tag, BoundsRect, GlobalPosition, GuiGraph, NodeId, Placement,
    WidgetBuilder, ZDepth,
};
use crate::comp::{Tag, Transform};
use crate::graphics::GraphicContext;
use specs::prelude::*;

pub fn create_frame(world: &mut World) -> Entity {
    create_container(world, layout::PackMode::Frame)
}

pub fn create_vbox(world: &mut World) -> Entity {
    create_container(world, layout::PackMode::Vertical)
}

pub fn create_hbox(world: &mut World) -> Entity {
    create_container(world, layout::PackMode::Horizontal)
}

pub fn create_container(world: &mut World, pack_mode: layout::PackMode) -> Entity {
    let mut pack = layout::Pack::new(pack_mode);
    pack.margin = [10.0, 10.0];

    world
        .create_entity()
        .with(Container)
        .with(Placement::zero())
        .with(pack)
        .with(GlobalPosition::new(0., 0.))
        .with(ZDepth::default())
        .with(Transform::default().with_position([0.0, 0.0, 0.0]))
        .with(BoundsRect::new(100.0, 100.0))
        .build()
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Container;

impl Container {
    pub fn vbox() -> ContainerBuilder {
        ContainerBuilder {
            parent_id: None,
            tag: None,
            pack_mode: layout::PackMode::Vertical,
            margin: [0.0, 0.0],
        }
    }

    pub fn hbox() -> ContainerBuilder {
        ContainerBuilder {
            parent_id: None,
            tag: None,
            pack_mode: layout::PackMode::Horizontal,
            margin: [0.0, 0.0],
        }
    }
}

pub struct ContainerBuilder {
    parent_id: Option<NodeId>,
    tag: Option<Tag>,
    pack_mode: layout::PackMode,
    margin: [f32; 2],
}

impl ContainerBuilder {
    pub fn child_of(mut self, parent_id: NodeId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_tag<S>(mut self, name: S) -> Self
    where
        S: ToString,
    {
        self.tag = Some(Tag::new(name));
        self
    }

    pub fn with_margin(mut self, margin: [f32; 2]) -> Self {
        self.margin = margin;
        self
    }
}

impl WidgetBuilder for ContainerBuilder {
    fn build(self, world: &mut World, _graphics: &mut GraphicContext) -> (Entity, NodeId) {
        let ContainerBuilder {
            parent_id,
            tag,
            pack_mode,
            margin,
        } = self;

        let mut pack = layout::Pack::new(pack_mode);
        pack.margin = margin;

        let entity_id = world
            .create_entity()
            .with(Container)
            .with(tag.unwrap_or_else(next_widget_tag))
            .with(Placement::zero())
            .with(pack)
            .with(GlobalPosition::new(0., 0.))
            .with(ZDepth::default())
            .with(Transform::default())
            .with(BoundsRect::new(100.0, 64.0))
            .build();

        let node_id = world
            .write_resource::<GuiGraph>()
            .insert_entity(entity_id, parent_id);

        (entity_id, node_id)
    }
}
