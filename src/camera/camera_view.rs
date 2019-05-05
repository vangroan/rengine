use nalgebra::{Matrix4, Point3, Unit, Vector3};
use specs::{Component, DenseVecStorage};

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct CameraView {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Unit<Vector3<f32>>,
}

impl CameraView {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn position(&self) -> &Point3<f32> {
        &self.eye
    }

    #[inline]
    pub fn set_position(&mut self, position: Point3<f32>) {
        self.eye = position;
    }

    #[inline]
    pub fn target(&self) -> &Point3<f32> {
        &self.target
    }

    #[inline]
    pub fn look_at(&mut self, target: Point3<f32>) {
        self.target = target;
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        // Right handed matrix must be used with perspective or orthographic projections
        let face_towards = Matrix4::look_at_rh(&self.eye, &self.target, &self.up);

        face_towards
    }
}

impl Default for CameraView {
    fn default() -> Self {
        CameraView {
            eye: Point3::new(0., 0., 0.),
            target: Point3::new(0., 0., -1.),
            up: Vector3::y_axis(),
        }
    }
}
