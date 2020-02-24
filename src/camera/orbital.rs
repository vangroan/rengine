use super::{ActiveCamera, CameraView};
use crate::angle::Rad;
use crate::option::lift2;
use crate::res::DeviceDimensions;
use glutin::{dpi::PhysicalPosition, ElementState, Event};
use nalgebra::{Point3, Rotation3, Unit, UnitQuaternion, Vector3};
use specs::{Component, DenseVecStorage, Read, ReadExpect, ReadStorage, System, WriteStorage};

/// Marks a camera to have arcball rotation controls.
#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct OrbitalCamera {
    /// Value between 0.0 and 1.0 to control the deceleration
    stop_ease: f32,

    /// Denominator for rotate speed.
    ///
    /// Number of physical pixels is divided by this value to
    /// determine the number of radians to rotate by.
    ///
    /// Lower values will result in faster rotation.
    ///
    /// Zero will cause a divide-by-zero panic.
    rotate_speed: f32,
}

impl OrbitalCamera {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn stop_ease(&self) -> f32 {
        self.stop_ease
    }

    #[inline]
    pub fn rotate_speed(&self) -> f32 {
        self.rotate_speed
    }
}

impl Default for OrbitalCamera {
    fn default() -> Self {
        OrbitalCamera {
            stop_ease: 0.9,
            rotate_speed: 1024.0,
        }
    }
}

/// System that takes user input and
/// applies it to all camera entities
/// marked for orbital controls.
pub struct OrbitalCameraControlSystem {
    last_cursor_pos: Option<PhysicalPosition>,
    cursor_diff: [f32; 2],
    input_state: ElementState,
}

#[derive(SystemData)]
pub struct OrbitalCameraControlSystemData<'a>(
    Read<'a, Vec<Event>>,
    ReadExpect<'a, DeviceDimensions>,
    Read<'a, ActiveCamera>,
    WriteStorage<'a, CameraView>,
    ReadStorage<'a, OrbitalCamera>,
);

impl OrbitalCameraControlSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for OrbitalCameraControlSystem {
    fn default() -> Self {
        OrbitalCameraControlSystem {
            last_cursor_pos: None,
            cursor_diff: [0.0, 0.0],
            input_state: ElementState::Released,
        }
    }
}

impl<'a> System<'a> for OrbitalCameraControlSystem {
    type SystemData = OrbitalCameraControlSystemData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        use glutin::{Event::*, MouseButton, WindowEvent::*};

        let OrbitalCameraControlSystemData(
            events,
            device_dim,
            active_camera,
            mut camera_views,
            orbital_cameras,
        ) = data;

        let mut cursor_still = true;

        for ev in events.iter() {
            if let WindowEvent { event, .. } = ev {
                match event {
                    CursorMoved { position, .. } => {
                        let current_pos = position.to_physical(device_dim.dpi_factor());
                        if let Some(last_pos) = self.last_cursor_pos.take() {
                            self.cursor_diff = [
                                (current_pos.x - last_pos.x) as f32,
                                (current_pos.y - last_pos.y) as f32,
                            ];
                        }
                        self.last_cursor_pos = Some(current_pos);
                        cursor_still = false;
                    }
                    MouseInput { state, button, .. } => {
                        if button == &MouseButton::Middle {
                            match state {
                                ElementState::Pressed => self.input_state = ElementState::Pressed,
                                ElementState::Released => {
                                    self.input_state = ElementState::Released;

                                    // Clear difference so next time user
                                    // rotates, the camera doesn't move
                                    // unexpectedly due to stale state.
                                    self.cursor_diff = [0.0, 0.0];
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Avoid looking up components if there's no rotation.
        let has_diff =
            self.cursor_diff[0] > ::std::f32::EPSILON || self.cursor_diff[1] > ::std::f32::EPSILON;

        // There is no event for when the cursor stops.
        //
        // The user may have stopped mouse movement, but kept the button
        // pressed. The camera would drift off according to the last stored
        // delta.
        //
        // Thus we track whether the cursor has moved, and taper off the delta
        // so the camera will come to a stop.
        if has_diff && (cursor_still || self.input_state == ElementState::Released) {
            let maybe_orbital = active_camera
                .camera_entity()
                .and_then(|e| orbital_cameras.get(e));

            if let Some(orbital) = maybe_orbital {
                self.cursor_diff = [
                    self.cursor_diff[0] * orbital.stop_ease,
                    self.cursor_diff[1] * orbital.stop_ease,
                ];
            }
        }

        if self.input_state == ElementState::Pressed {
            let maybe_camera = active_camera.camera_entity().and_then(|e| {
                lift2(
                    camera_views.get_mut(e),
                    orbital_cameras.get(e), // Only move cameras marked as orbital
                )
            });

            if let Some((mut view, orbit)) = maybe_camera {
                arcball_rotate(
                    &mut view,
                    Rad(self.cursor_diff[1] / orbit.rotate_speed),
                    Rad(-self.cursor_diff[0] / orbit.rotate_speed), // Flip yaw for more intuitive interface
                );
            }
        }
    }
}

pub fn arcball_rotate(camera_view: &mut CameraView, pitch: Rad<f32>, yaw: Rad<f32>) {
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
    let new_focus_dir = pitch_rot.transform_vector(&yaw_rot.transform_vector(&focus));
    let new_focus = new_focus_dir * camera_distance;

    // Focus vector is the difference between the camera position and target
    // position. It is thus from the reference of the target.
    //
    // Camera position is in global space. The target position needs to be
    // added back, otherwise the camera would be placed in local space relative
    // to the target position.
    let new_pos: Point3<f32> = camera_view.target() + new_focus;
    camera_view.set_position(new_pos);
}
