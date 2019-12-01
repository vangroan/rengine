use super::CameraView;
use nalgebra::{Point3, Vector3};
use specs::{Component, DenseVecStorage, Join, ReadStorage, System, WriteStorage};

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct FocusTarget(Point3<f32>);

impl FocusTarget {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_target<P>(pos: P) -> Self
    where
        P: Into<Point3<f32>>,
    {
        FocusTarget(pos.into())
    }

    #[inline]
    pub fn position(&self) -> &Point3<f32> {
        &self.0
    }

    #[inline]
    pub fn set_position<P>(&mut self, pos: P)
    where
        P: Into<Point3<f32>>,
    {
        self.0 = pos.into();
    }
}

impl Default for FocusTarget {
    fn default() -> Self {
        FocusTarget(Point3::new(0.0, 0.0, 0.0))
    }
}

/// Interpolates camera views towards their focus targets.
#[derive(Debug, Default)]
pub struct CameraDriftSystem;

impl CameraDriftSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> System<'a> for CameraDriftSystem {
    type SystemData = (WriteStorage<'a, CameraView>, ReadStorage<'a, FocusTarget>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut camera_views, focus_targets) = data;

        for (camera_view, focus_target) in (&mut camera_views, &focus_targets).join() {
            let proximity = (camera_view.target() - focus_target.0).magnitude();

            // Is camera at rest?
            if proximity > ::std::f32::EPSILON {
                // Tri-linear interpolate towards grid camera target
                let time = 0.50;
                let new_target =
                    camera_view.target() + ((focus_target.0 - camera_view.target()) * time);
                // Both camera and target positions will be shifted.
                let camera_diff: Vector3<f32> = camera_view.position() - camera_view.target();
                camera_view.set_position(new_target + camera_diff);
                camera_view.look_at(new_target);
            }
        }
    }
}
