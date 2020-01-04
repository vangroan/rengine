use super::{ActiveCamera, CameraView};
use crate::option::lift2;
use crate::res::DeltaTime;
use glutin::Event;
use nalgebra::Vector3;
use specs::{Component, DenseVecStorage, Read, ReadStorage, System, WriteStorage};
use std::time::Duration;

/// Marks a camera with controls to move closer and further.
#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct DollyCamera {
    speed: f32,
}

impl DollyCamera {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for DollyCamera {
    fn default() -> Self {
        DollyCamera { speed: 100.0 }
    }
}

#[derive(Debug, Default)]
pub struct DollyCameraControlSystem;

impl DollyCameraControlSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> System<'a> for DollyCameraControlSystem {
    type SystemData = (
        Read<'a, DeltaTime>,
        Read<'a, Vec<Event>>,
        Read<'a, ActiveCamera>,
        WriteStorage<'a, CameraView>,
        ReadStorage<'a, DollyCamera>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use glutin::{Event::*, MouseScrollDelta::*, TouchPhase, WindowEvent::*};

        let (dt, events, active_camera, mut camera_views, dolly_cameras) = data;
        let mut movement = 0.0;

        for ev in events.iter() {
            match ev {
                WindowEvent { event, .. } => match event {
                    MouseWheel { delta, phase, .. } => {
                        if phase == &TouchPhase::Moved {
                            // Currently only support mouse and not touchpad.
                            if let LineDelta(_x, y) = delta {
                                // Mouse wheel increases on up (away from user)
                                // and decreases on down (towards user).
                                //
                                // Flip the sign so dolly closer is decrease and
                                // dolly further is increase.
                                movement = -y;
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // Approximately not floating point zero.
        if movement > ::std::f32::EPSILON || movement < -::std::f32::EPSILON {
            let maybe_camera = active_camera.camera_entity().and_then(|e| {
                lift2(
                    camera_views.get_mut(e),
                    dolly_cameras.get(e), // Only dolly cameras considered
                )
            });

            if let Some((camera_view, _dolly_camera)) = maybe_camera {
                dolly_move(camera_view, movement, 100.0, dt.duration());
            }
        }
    }
}

pub fn dolly_move(camera_view: &mut CameraView, movement: f32, speed: f32, dt: &Duration) {
    let camera_diff: Vector3<f32> = camera_view.position() - camera_view.target();

    // New distance is old distance with the movement added
    let camera_distance = camera_diff.magnitude();

    // Vector pointing from focus target to camera position.
    let focus: Vector3<f32> = camera_diff.normalize();

    let new_distance = camera_distance + (movement * speed * (dt.as_millis() as f32 / 1000.0));
    let new_focus = focus * new_distance;

    // Focus is the difference between camera position and
    // target position, from the target as a reference.
    //
    // Position needs to be translated by target position
    // in order to get camera position in global space.
    camera_view.set_position(camera_view.target() + new_focus);
}
