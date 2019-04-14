/// View port rectangle used for communicating target window size to
/// rendering systems.
///
/// The view port is concerned with the physical size of the device.
///
/// see [Gfx, windows, and resizing](https://falseidolfactory.com/2018/05/28/gfx-windows-and-resizing.html)
#[derive(Debug)]
pub struct ViewPort {
    pub(crate) rect: gfx::Rect,
}

impl ViewPort {
    /// Create a view port rectangle covering the desired device target
    pub fn new(device_size: (u16, u16)) -> Self {
        ViewPort {
            rect: gfx::Rect {
                x: 0,
                y: 0,
                w: device_size.0,
                h: device_size.1,
            },
        }
    }
}
