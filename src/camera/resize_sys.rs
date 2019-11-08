use crate::camera::CameraProjection;
use crate::res::DeviceDimensions;
use specs::{Join, Read, System, WriteStorage};

/// Update all cameras on window resize events.
///
/// This is required so that the world view does not distort when
/// the window is stretched.
pub struct CameraResizeSystem;

impl CameraResizeSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for CameraResizeSystem {
    fn default() -> Self {
        CameraResizeSystem
    }
}

impl<'a> System<'a> for CameraResizeSystem {
    type SystemData = (
        Read<'a, DeviceDimensions>,
        WriteStorage<'a, CameraProjection>,
    );

    fn run(&mut self, (dim, mut cam_views): Self::SystemData) {
        let (dev_w, dev_h): (u32, u32) = dim.logical_size.into();

        for (ref mut view,) in (&mut cam_views,).join() {
            view.set_device_size((dev_w as u16, dev_h as u16));
        }
    }
}
