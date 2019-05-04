use nalgebra::{Matrix4, Point3, Unit, Vector3, Vector4};
use specs::{Component, DenseVecStorage};

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct CameraView {
    position: Point3<f32>,
    target: Point3<f32>,
    up: Unit<Vector3<f32>>,
    forward: Unit<Vector3<f32>>,
    right: Unit<Vector3<f32>>,
}

impl CameraView {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn position(&self) -> &Point3<f32> {
        &self.position
    }

    #[inline]
    pub fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
        self.update_right();
    }

    #[inline]
    pub fn target(&self) -> &Point3<f32> {
        &self.target
    }

    pub fn look_at(&mut self, target: Point3<f32>) {
        self.target = target;
        self.update_right();
    }

    fn update_right(&mut self) {
        self.right = Unit::new_normalize(self.forward.as_ref().cross(self.up.as_ref()));
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        // Right handed matrix must be used with perspective or orthographic projections
        let face_towards = Matrix4::look_at_rh(&self.position, &self.target, &self.up);

        face_towards
    }
}

impl Default for CameraView {
    fn default() -> Self {
        CameraView {
            position: Point3::new(0., 0., 0.),
            target: Point3::new(0., 0., -1.),
            up: Vector3::y_axis(),
            forward: -Vector3::z_axis(),
            right: Vector3::x_axis(),
        }
    }
}
