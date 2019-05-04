use crate::angle::Deg;
use nalgebra::{Matrix4, Point3};
use specs::{Component, DenseVecStorage};

const DEFAULT_SCALE_PIXELS: f32 = 1000.;

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct CameraProjection {
    near: f32,
    far: f32,
    scale_pixels: f32,
    device_size: [u16; 2],
    fovy: Deg<f32>,
    aspect_ratio: f32,
}

impl CameraProjection {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_device_size(device_size: (u16, u16)) -> Self {
        let mut camera_proj = CameraProjection::new();
        camera_proj.set_device_size(device_size);
        camera_proj
    }

    pub fn set_device_size(&mut self, device_size: (u16, u16)) {
        self.device_size = [device_size.0, device_size.1];
        if device_size.1 != 0 {
            self.aspect_ratio = device_size.0 as f32 / device_size.1 as f32;
        }
    }

    pub fn proj_matrix<V>(&self, position: V) -> Matrix4<f32>
    where
        V: Into<Point3<f32>>,
    {
        let near = self.near;
        let far = self.far;
        let pos = position.into();
        let [dev_w, dev_h] = self.device_size;
        let scale_pixels = self.scale_pixels;
        let (width, height) = (dev_w as f32 / scale_pixels, dev_h as f32 / scale_pixels);
        let (x, y) = (pos.x - (width / 2.), pos.y - (height / 2.));

        Matrix4::new_orthographic(x, x + width, y, y + height, near, far)
    }

    pub fn prespective_matrix(&self) -> Matrix4<f32> {
        let near = self.near;
        let far = self.far;
        let fovy = self.fovy.as_radians();
        let aspect = self.aspect_ratio;

        Matrix4::new_perspective(aspect, fovy, near, far)
    }
}

impl Default for CameraProjection {
    fn default() -> Self {
        CameraProjection {
            near: 0.1,
            far: 1000.,
            scale_pixels: DEFAULT_SCALE_PIXELS,
            device_size: [0, 0],
            fovy: Deg(10.),
            aspect_ratio: 16. / 9.,
        }
    }
}
