use super::super::{PackMode, Placement, WidgetBounds};
use crate::comp::Transform;
use specs::{Builder, Component, DenseVecStorage, Entity, World};

pub fn create_frame(world: &mut World) -> Entity {
    world
        .create_entity()
        .with(Container)
        .with(Placement::zero())
        .with(PackMode::Frame)
        .with(Transform::default().with_position([0.0, 0.0, 0.0]))
        .with(WidgetBounds::new(100.0, 100.0))
        .build()
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Container;
