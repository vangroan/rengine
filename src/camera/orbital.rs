use super::{ActiveCamera, CameraProjection, CameraView};
use crate::angle::Rad;
use crate::option::lift3;
use crate::res::DeviceDimensions;
use glutin::{dpi::PhysicalPosition, Event};
use nalgebra::{Rotation3, Unit, UnitQuaternion, Vector3};
use specs::{
    Component, DenseVecStorage, Join, Read, ReadExpect, ReadStorage, System, WriteStorage,
};

#[derive(Component, Debug, Default)]
#[storage(DenseVecStorage)]
/// Marks a camera to have arcball rotation controls.
pub struct OrbitalCamera;

impl OrbitalCamera {
    pub fn new() -> Self {
        Default::default()
    }
}

/// System that takes user input and
/// applies it to all camera entities
/// marked for orbital controls.
#[derive(Default)]
pub struct OrbitalCameraControlSystem {
    last_cursor_pos: Option<PhysicalPosition>,
    cursor_diff: [f32; 2],
    orbit_state: bool,
}

impl OrbitalCameraControlSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> System<'a> for OrbitalCameraControlSystem {
    type SystemData = (
        Read<'a, Vec<Event>>,
        ReadExpect<'a, DeviceDimensions>,
        Read<'a, ActiveCamera>,
        ReadStorage<'a, CameraProjection>,
        WriteStorage<'a, CameraView>,
        ReadStorage<'a, OrbitalCamera>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use glutin::{ElementState, Event::*, MouseButton, WindowEvent::*};

        let (events, device_dim, active_camera, camera_projs, mut camera_views, orbital_cameras) =
            data;

        for ev in events.iter() {
            match ev {
                WindowEvent { event, .. } => match event {
                    CursorMoved { position, .. } => {
                        let current_pos = position.to_physical(device_dim.dpi_factor());
                        if let Some(last_pos) = self.last_cursor_pos.take() {
                            self.cursor_diff = [
                                (current_pos.x - last_pos.x) as f32,
                                (current_pos.y - last_pos.y) as f32,
                            ];
                        }
                        self.last_cursor_pos = Some(current_pos);
                    }
                    MouseInput { state, button, .. } => {
                        if button == &MouseButton::Middle {
                            match state {
                                ElementState::Pressed => self.orbit_state = true,
                                ElementState::Released => self.orbit_state = false,
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if self.orbit_state {
            let maybe_camera = active_camera.camera_entity().and_then(|e| {
                lift3(
                    camera_projs.get(e),
                    camera_views.get_mut(e),
                    orbital_cameras.get(e),
                )
            });

            if let Some((proj, mut view, orbit)) = maybe_camera {
                arcball_rotate(
                    (&mut view,),
                    Rad(self.cursor_diff[1] / 1024.0),
                    Rad(self.cursor_diff[0] / 1024.0),
                );
            }
        }
    }
}

pub fn arcball_rotate(data: (&mut CameraView,), pitch: Rad<f32>, yaw: Rad<f32>) {
    println!("Rotate {} {}", pitch, yaw);
    let (camera_view,) = data;

    let camera_diff: Vector3<f32> = camera_view.position() - camera_view.target();

    // Keep the distance between the camera and target.
    let camera_distance = camera_diff.magnitude();

    // Vector pointing from focus target to camera position.
    let focus: Vector3<f32> = camera_diff.normalize();

    // Use normalised up vector as axis for yaw matrix
    let up: Unit<Vector3<f32>> = *camera_view.up();
    let yaw_rot =
        UnitQuaternion::from_rotation_matrix(&Rotation3::from_axis_angle(&up, yaw.as_radians()));

    // Use normalised right vector as axis for pitch matrix
    let right: Unit<Vector3<f32>> = camera_view.make_right();
    let pitch_rot = UnitQuaternion::from_rotation_matrix(&Rotation3::from_axis_angle(
        &right,
        pitch.as_radians(),
    ));

    // Rotate focus vector to determine new camera location.
    let new_focus = pitch_rot.transform_vector(&yaw_rot.transform_vector(&focus));
    camera_view.set_position((new_focus * camera_distance).into());
}
