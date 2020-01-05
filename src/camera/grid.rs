//! Camera control that locks the focus target on a voxel-axis in a 3D grid.

use super::{ActiveCamera, FocusTarget};
use crate::option::lift2;
use glutin::Event;
use nalgebra::Vector3;
use specs::{Component, DenseVecStorage, Read, System, WriteStorage};

/// Marks a camera with grid based control.
///
/// # FIXME
/// Because the system assigns the target in `GridCamera` to the
/// camera's target, the `GridCamera` becomes the source of truth
/// for the camera's target.
///
/// Logic needs to change so that `GridCamera` does not overrule
/// other systems that are changing camera look-at.
#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct GridCamera;

impl GridCamera {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for GridCamera {
    fn default() -> Self {
        GridCamera
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
        WriteStorage<'a, FocusTarget>,
        WriteStorage<'a, GridCamera>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use glutin::{ElementState, Event::*, VirtualKeyCode, WindowEvent::*};

        let (events, active_camera, mut focus_targets, mut grid_cameras) = data;
        let mut offset: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);

        for ev in events.iter() {
            if let WindowEvent { event, .. } = ev {
                if let KeyboardInput { input, .. } = event {
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
            }
        }

        // Apply input to active grid camera.
        if offset.y > ::std::f32::EPSILON || offset.y < -::std::f32::EPSILON {
            let maybe_camera = active_camera.camera_entity().and_then(|e| {
                lift2(
                    focus_targets.get_mut(e),
                    grid_cameras.get_mut(e), // Only grid cameras
                )
            });

            if let Some((focus_target, _grid_camera)) = maybe_camera {
                focus_target.set_position(focus_target.position() + offset);
            }
        }
    }
}
