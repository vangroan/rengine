use nalgebra::Matrix4;

/// View port rectangle used for communicating target window size to
/// rendering systems.
///
/// see [Gfx, windows, and resizing](https://falseidolfactory.com/2018/05/28/gfx-windows-and-resizing.html)
#[derive(Debug)]
pub struct ViewPort {
    pub(crate) rect: gfx::Rect,
    pub(crate) matrix: Matrix4<f32>,
}

impl ViewPort {
    /// Create a view port rectangle covering the desired device target
    pub fn new(device_size: (u16, u16)) -> Self {
        let (dev_w, dev_h) = device_size;

        // TODO: Does DPI need to be taken into account?
        let scale_factor = 1000.;

        // TODO: Refactor into Orthographic Camera component
        let matrix = Matrix4::new_orthographic(
            0.,
            dev_w as f32 / scale_factor,
            0.,
            dev_h as f32 / scale_factor,
            -1.0,
            1.0,
        );

        ViewPort {
            rect: gfx::Rect {
                x: 0,
                y: 0,
                w: device_size.0,
                h: device_size.1,
            },
            matrix,
        }
    }
}
