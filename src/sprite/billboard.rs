use crate::camera::ActiveCamera;
use crate::comp::Transform;
use specs::{Component, FlaggedStorage, Join, ReadExpect, ReadStorage, System, WriteStorage};

#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct Billboard;

pub struct BillboardSystem;

impl<'a> System<'a> for BillboardSystem {
    type SystemData = (
        ReadExpect<'a, ActiveCamera>,
        ReadStorage<'a, Billboard>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (active_camera, billboards, mut transforms) = data;

        for (ref _billboard, ref mut transform) in (&billboards, &transforms).join() {
            // TODO: Orient billboards towards camera
        }
    }
}
