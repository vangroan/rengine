//! Camera control that locks the focus target on a voxel-axis in a 3D grid.

use super::{ActiveCamera, CameraView};
use glutin::Event;
use nalgebra::{Point3, Vector3};
use specs::{Component, DenseVecStorage, Join, Read, System, WriteStorage};

/// Marks a camera with grid based control.
///
/// FIXME: Because the system assigns the target in `GridCamera` to the
///        camera's target, the `GridCamera` becomes the source of truth
///        for the camera's target.
///
///        Logic needs to change so that `GridCamera` does not overrule
///        other systems that are changing camera look-at.
#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct GridCamera {
    target: Point3<f32>,
}

impl GridCamera {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_target<P: Into<Point3<f32>>>(target: P) -> Self {
        GridCamera {
            target: target.into(),
        }
    }
}

impl Default for GridCamera {
    fn default() -> Self {
        GridCamera {
            target: Point3::new(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Default)]
pub struct GridCameraControlSystem;

impl GridCameraControlSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> System<'a> for GridCameraControlSystem {
    type SystemData = (
        Read<'a, Vec<Event>>,
        Read<'a, ActiveCamera>,
        WriteStorage<'a, CameraView>,
        WriteStorage<'a, GridCamera>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use glutin::{ElementState, Event::*, VirtualKeyCode, WindowEvent::*};

        let (events, active_camera, mut camera_views, mut grid_cameras) = data;
        let mut offset: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);

        for ev in events.iter() {
            match ev {
                WindowEvent { event, .. } => match event {
                    KeyboardInput { input, .. } => {
                        if input.state == ElementState::Released {
                            if let Some(key_code) = input.virtual_keycode {
                                match key_code {
                                    VirtualKeyCode::PageUp => offset.y = 1.0,
                                    VirtualKeyCode::PageDown => offset.y = -1.0,
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // Apply input to active grid camera.
        if offset.y > ::std::f32::EPSILON || offset.y < -::std::f32::EPSILON {
            let maybe_camera = active_camera
                .camera_entity()
                .and_then(|e| grid_cameras.get_mut(e));

            if let Some(grid_camera) = maybe_camera {
                grid_camera.target += offset;
            }
        }

        // Apply movement to all grid cameras.
        for (camera_view, grid_camera) in (&mut camera_views, &grid_cameras).join() {
            let proximity = (camera_view.target() - grid_camera.target).magnitude();

            // Is camera at rest?
            if proximity > ::std::f32::EPSILON {
                // Tri-linear interpolate towards grid camera target
                let time = 0.50;
                let new_target =
                    camera_view.target() + ((grid_camera.target - camera_view.target()) * time);
                // Both camera and target positions will be shifted.
                let camera_diff: Vector3<f32> = camera_view.position() - camera_view.target();
                camera_view.set_position(new_target + camera_diff);
                camera_view.look_at(new_target);
            }
        }
    }
}
