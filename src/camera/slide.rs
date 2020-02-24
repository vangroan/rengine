//! Camera control that moves a camera along the x-z plane.
//!
//! Forward is defined as from the camera position to the focus target position.

use super::{ActiveCamera, CameraView, FocusTarget};
use crate::option::lift3;
use crate::res::{DeltaTime, DeviceDimensions};
use glutin::{dpi::LogicalPosition, Event};
use nalgebra::Vector3;
use specs::{Component, DenseVecStorage, Read, ReadStorage, System, WriteStorage};

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct SlideCamera {
    speed: f32,
}

impl SlideCamera {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for SlideCamera {
    fn default() -> Self {
        SlideCamera { speed: 10.0 }
    }
}

#[derive(Default)]
pub struct SlideCameraControlSystem {
    cursor_pos: Option<LogicalPosition>,
}

#[derive(SystemData)]
pub struct SlideCameraControlSystemData<'a>(
    Read<'a, Vec<Event>>,
    Read<'a, DeviceDimensions>,
    Read<'a, DeltaTime>,
    Read<'a, ActiveCamera>,
    ReadStorage<'a, CameraView>,
    WriteStorage<'a, FocusTarget>,
    ReadStorage<'a, SlideCamera>,
);

impl SlideCameraControlSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> System<'a> for SlideCameraControlSystem {
    type SystemData = SlideCameraControlSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        use glutin::{Event::*, WindowEvent::*};

        let SlideCameraControlSystemData(
            events,
            device_dim,
            dt,
            active_camera,
            camera_views,
            mut focus_targets,
            slide_cameras,
        ) = data;

        for ev in events.iter() {
            if let WindowEvent { event, .. } = ev {
                if let CursorMoved { position, .. } = event {
                    self.cursor_pos = Some(*position);
                }
            }
        }

        if let Some(pos) = self.cursor_pos {
            let device_logical_size = device_dim.logical_size();
            let mut dir = [0.0; 2];
            let threshold = 50.0;

            // Left
            if pos.x < threshold {
                dir[0] -= 1.0;
            }

            // Right
            if pos.x > device_logical_size.width - threshold {
                dir[0] += 1.0;
            }

            // Up
            if pos.y > device_logical_size.height - threshold {
                dir[1] -= 1.0;
            }

            // Down
            if pos.y < threshold {
                dir[1] += 1.0;
            }

            let maybe_camera = active_camera.camera_entity().and_then(|e| {
                lift3(
                    camera_views.get(e),
                    focus_targets.get_mut(e),
                    slide_cameras.get(e), // Only slide cameras
                )
            });

            if let Some((camera_view, focus_target, slide_camera)) = maybe_camera {
                let camera_diff = camera_view.target() - camera_view.position();

                // Strip Y coordinate so movement is only on x-z plane.camera_diff
                let forward = {
                    let mut f = camera_diff;
                    f.y = 0.0;
                    f.normalize()
                };

                let up: Vector3<f32> = Vector3::y_axis().into_inner();
                let right = forward.cross(&up);

                let focus_position = focus_target.position();
                let new_position = focus_position
                    + (forward * dir[1] * slide_camera.speed * dt.as_secs_float())
                    + (right * dir[0] * slide_camera.speed * dt.as_secs_float());

                focus_target.set_position(new_position);
            }
        }
    }
}
