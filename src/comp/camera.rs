use nalgebra::{Matrix4, Point3};
use specs::{Component, DenseVecStorage};

const DEFAULT_SCALE_PIXELS: f32 = 1000.;
const UP_AXIS: [f32; 3] = [0., 1., 0.];

/// Camera keeps projection information.
///
/// Camera needs to know of device size in logical pixels.
#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Camera {
    /// The number of physical device pixels that spans a screen unit
    scale_pixels: f32,
    device_size: [u16; 2],

    pub(crate) target_pos: Point3<f32>,
    pub(crate) proj_matrix: Matrix4<f32>,
}

impl Camera {
    /// Creates a new camera.
    ///
    /// `device_size` is the initial logical dimensions of the device.
    pub fn with_device_size(device_size: (u16, u16)) -> Self {
        let mut camera = Camera {
            scale_pixels: DEFAULT_SCALE_PIXELS,
            device_size: [0, 0],
            target_pos: Point3::new(0., 0., 0.),
            proj_matrix: Matrix4::identity(),
        };

        camera.update_view(device_size);

        camera
    }

    /// Notify the camera that the view port dimensions have udpated.
    pub fn update_view(&mut self, device_size: (u16, u16)) {
        self.device_size = [device_size.0, device_size.1];
        // let (dev_w, dev_h) = device_size;
        // let scale_pixels = self.scale_pixels;

        // self.proj_matrix = Matrix4::new_orthographic(
        //     -1.,
        //     dev_w as f32 / scale_pixels,
        //     -1.,
        //     dev_h as f32 / scale_pixels,
        //     -10.0,
        //     10.0,
        // );
    }

    pub fn target(&self) -> &Point3<f32> {
        &self.target_pos
    }

    pub fn set_target<V>(&mut self, pos: V)
    where
        V: Into<Point3<f32>>,
    {
        self.target_pos = pos.into();
    }

    pub fn view_matrix<V>(&self, camera_pos: V) -> Matrix4<f32>
    where
        V: Into<Point3<f32>>,
    {
        let camera_pos_point = camera_pos.into();

        // Target is relative to implicit eye position, which is [0, 0, 0]
        let target_pos = camera_pos_point.to_homogeneous() + self.target_pos.to_homogeneous();
        Matrix4::face_towards(
            &camera_pos_point,
            &Point3::from_homogeneous(target_pos).unwrap(),
            &UP_AXIS.into(),
        )
    }

    pub fn proj_matrix<V>(&self, camera_pos: V) -> Matrix4<f32>
    where
        V: Into<Point3<f32>>,
    {
        let pos = camera_pos.into();
        let [dev_w, dev_h] = self.device_size;
        let scale_pixels = self.scale_pixels;
        let (width, height) = (dev_w as f32 / scale_pixels, dev_h as f32 / scale_pixels);

        Matrix4::new_orthographic(pos.x, pos.x + width, pos.y, pos.y + height, -10.0, 10.0)
    }
}

/// Camera Projection
#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct CameraProj {}
