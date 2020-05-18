use super::super::{layout, Placement, WidgetBounds};
use crate::comp::Transform;
use specs::{Builder, Component, DenseVecStorage, Entity, World};

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
        .with(Transform::default().with_position([0.0, 0.0, 0.0]))
        .with(WidgetBounds::new(100.0, 100.0))
        .build()
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Container;
