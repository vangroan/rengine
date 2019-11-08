use crate::camera::{ActiveCamera, CameraView};
use crate::comp::Transform;
use glm;
use nalgebra::Unit;
use specs::{Component, FlaggedStorage, Join, ReadExpect, ReadStorage, System, WriteStorage};

#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct Billboard;

/// System that orients entities marked as billboards
/// towards the currently active camera.
pub struct BillboardSystem;

impl BillboardSystem {
    pub fn new() -> Self {
        BillboardSystem
    }
}

impl<'a> System<'a> for BillboardSystem {
    type SystemData = (
        ReadExpect<'a, ActiveCamera>,
        ReadStorage<'a, CameraView>,
        ReadStorage<'a, Billboard>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (active_camera, camera_views, billboards, mut transforms) = data;

        // Determine active camera
        let maybe_camera_view = active_camera
            .camera_entity()
            .and_then(|entity| camera_views.get(entity));

        if let Some(camera_view) = maybe_camera_view {
            for (ref _billboard, ref mut transform) in (&billboards, &mut transforms).join() {
                // TODO: Decide on either nalgebra or nalgebra_glm, but not both.

                // Convert up from Vector3 to Vector4
                let up = {
                    let cam_up = camera_view.up();
                    glm::Vec3::new(cam_up.x, cam_up.y, cam_up.z)
                };
                let diff = (camera_view.eye() - transform.pos).to_homogeneous();
                let dir = Unit::new_normalize(diff);

                transform.look_at(glm::Vec3::new(dir.x, dir.y, dir.z), up);
            }
        }
    }
}
