use super::super::{
    layout, next_widget_tag, BoundsRect, GlobalPosition, GuiGraph, NodeId, Placement,
    WidgetBuilder, ZDepth,
};
use crate::comp::{Tag, Transform};
use crate::graphics::GraphicContext;
use specs::prelude::*;

/// Creates a container widget without inserting it into a GUI graph.
///
/// Useful for creating the initial root widget.
pub fn create_container(world: &mut World, pack_mode: layout::PackMode) -> Entity {
    let mut pack = layout::Pack::new(pack_mode);
    pack.margin = [10.0, 10.0];

    world
        .create_entity()
        .with(Container)
        .with(next_widget_tag())
        .with(Placement::zero())
        .with(pack)
        .with(GlobalPosition::new(0., 0.))
        .with(ZDepth::default())
        .with(Transform::default())
        .with(BoundsRect::new(::std::f32::INFINITY, ::std::f32::INFINITY))
        .build()
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Container;

impl Container {
    pub fn frame() -> ContainerBuilder {
        ContainerBuilder {
            pack_mode: layout::PackMode::Frame,
            ..ContainerBuilder::default()
        }
    }

    pub fn vbox() -> ContainerBuilder {
        ContainerBuilder {
            pack_mode: layout::PackMode::Vertical,
            ..ContainerBuilder::default()
        }
    }

    pub fn hbox() -> ContainerBuilder {
        ContainerBuilder {
            pack_mode: layout::PackMode::Horizontal,
            ..ContainerBuilder::default()
        }
    }
}

pub struct ContainerBuilder {
    parent_id: Option<NodeId>,
    tag: Option<Tag>,
    placement: layout::Placement,
    pack_mode: layout::PackMode,
    margin: [f32; 2],
    size: [f32; 2],
}

impl Default for ContainerBuilder {
    fn default() -> Self {
        ContainerBuilder {
            parent_id: None,
            tag: None,
            placement: layout::Placement::zero(),
            pack_mode: layout::PackMode::Frame,
            margin: [0.0, 0.0],
            size: [::std::f32::INFINITY, ::std::f32::INFINITY],
        }
    }
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

    pub fn with_placement(mut self, offset: [f32; 2]) -> Self {
        self.placement = layout::Placement::new(offset[0], offset[1]);
        self
    }

    pub fn with_margin(mut self, margin: [f32; 2]) -> Self {
        self.margin = margin;
        self
    }

    pub fn with_size(mut self, size: [f32; 2]) -> Self {
        self.size = size;
        self
    }
}

impl WidgetBuilder for ContainerBuilder {
    fn build(self, world: &mut World, _graphics: &mut GraphicContext) -> (Entity, NodeId) {
        let ContainerBuilder {
            parent_id,
            tag,
            placement,
            pack_mode,
            margin,
            size,
        } = self;

        let mut pack = layout::Pack::new(pack_mode);
        pack.margin = margin;

        let entity_id = world
            .create_entity()
            .with(Container)
            .with(tag.unwrap_or_else(next_widget_tag))
            .with(placement)
            .with(pack)
            .with(GlobalPosition::new(0., 0.))
            .with(ZDepth::default())
            .with(Transform::default())
            .with(BoundsRect::new(size[0], size[1]))
            .build();

        let node_id = world
            .write_resource::<GuiGraph>()
            .insert_entity(entity_id, parent_id);

        (entity_id, node_id)
    }
}
