use glm::Mat4x4;

/// View port rectangle used for communicating target window size to
/// rendering systems.
///
/// see [Gfx, windows, and resizing](https://falseidolfactory.com/2018/05/28/gfx-windows-and-resizing.html)
#[derive(Debug)]
pub struct ViewPort {
    pub(crate) rect: gfx::Rect,
    // pub(crate) scale: Mat4x4,
}

impl ViewPort {
    /// Create a view port rectangle covering the desired device target
    pub fn new(device_size: (u16, u16)) -> Self {
        let (dev_w, dev_h) = device_size;

        // TODO: Transform

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
