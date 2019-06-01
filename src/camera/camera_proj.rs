use crate::angle::Deg;
use nalgebra::{Matrix4, Point3};
use specs::{Component, DenseVecStorage};

const DEFAULT_SCALE_PIXELS: f32 = 1000.;

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct CameraProjection {
    ortho: OrthographicSettings,
    persp: PerspectiveSettings,
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
        // Orthographic
        self.ortho.device_size = [device_size.0, device_size.1];

        // Perspective
        if device_size.1 != 0 {
            self.persp.aspect_ratio = device_size.0 as f32 / device_size.1 as f32;
        }
    }

    pub fn orthographic<V>(&self, position: V) -> Matrix4<f32>
    where
        V: Into<Point3<f32>>,
    {
        let near = self.ortho.nearz;
        let far = self.ortho.farz;
        let pos = position.into();
        let [dev_w, dev_h] = self.ortho.device_size;
        let scale_pixels = self.ortho.scale_pixels;

        let (width, height) = (dev_w as f32 / scale_pixels, dev_h as f32 / scale_pixels);
        let (x, y) = (pos.x - (width / 2.), pos.y - (height / 2.));

        Matrix4::new_orthographic(x, x + width, y, y + height, near, far)
    }

    pub fn perspective(&self) -> Matrix4<f32> {
        let near = self.persp.nearz;
        let far = self.persp.farz;
        let fovy = self.persp.fovy.as_radians();
        let aspect = self.persp.aspect_ratio;

        Matrix4::new_perspective(aspect, fovy, near, far)
    }

    pub fn perspective_settings(&self) -> &PerspectiveSettings {
        &self.persp
    }
}

impl Default for CameraProjection {
    fn default() -> Self {
        CameraProjection {
            ortho: OrthographicSettings {
                nearz: -10.0,
                farz: 10.0,
                scale_pixels: DEFAULT_SCALE_PIXELS,
                device_size: [0, 0],
            },
            persp: PerspectiveSettings {
                nearz: 0.1,
                farz: 1000.,
                fovy: Deg(10.),

                // Aspect ratio must never be 0
                aspect_ratio: 16. / 9.,
            },
        }
    }
}

#[derive(Debug)]
struct OrthographicSettings {
    nearz: f32,
    farz: f32,
    scale_pixels: f32,
    device_size: [u16; 2],
}

#[derive(Debug)]
pub struct PerspectiveSettings {
    nearz: f32,
    farz: f32,
    fovy: Deg<f32>,
    aspect_ratio: f32,
}

impl PerspectiveSettings {
    #[inline]
    pub fn nearz(&self) -> f32 {
        self.nearz
    }

    #[inline]
    pub fn farz(&self) -> f32 {
        self.farz
    }

    #[inline]
    pub fn fovy(&self) -> Deg<f32> {
        self.fovy
    }

    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }
}
