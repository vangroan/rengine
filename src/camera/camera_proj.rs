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

    pub fn prespective_matrix<V>(&self, _position: V) -> Matrix4<f32>
    where
        V: Into<Point3<f32>>,
    {
        unimplemented!()
    }
}

impl Default for CameraProjection {
    fn default() -> Self {
        CameraProjection {
            near: -10.,
            far: 10.,
            scale_pixels: DEFAULT_SCALE_PIXELS,
            device_size: [0, 0],
        }
    }
}
