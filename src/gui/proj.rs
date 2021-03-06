//! Projection matrix for GUI.
//!
//!
//! ```ignore
//!  -1,1 ------------------------ 1,1
//!      |                        |
//!      |                        |
//!      |                        |
//!      |                        |
//!      |          0,0           |
//!      |                        |
//!      |                        |
//!      |                        |
//!      |                        |
//! -1,-1 ------------------------ 1,-1
//! ```
//!
//! ```ignore
//!
//! ```
use glutin::dpi::{LogicalSize, PhysicalSize};
use nalgebra::{Matrix4, Vector3};

/// Create the view matrix of the GUI.
///
/// `pixel_scale` is the width and height in logical pixel size of a 1.0 by 1.0 quad.
///
/// TODO:
///   - Explain missing z coordinate
///   - Explain scale to cancel window stretch
///   - Explain translate by whole window size
pub fn create_gui_proj_matrix<P>(device_size: P, dpi_factor: f32) -> Matrix4<f32>
where
    P: Into<PhysicalSize>,
{
    let PhysicalSize {
        width: device_w,
        height: device_h,
    } = device_size.into();

    // The normalised device coordinates (-1 to 1) is 2 wide and high
    let normalised_device_width = 2.0;
    let scale_factor = normalised_device_width * dpi_factor;

    // z-axis is for depth and sorting
    let nearz = -65535.;
    let farz = 65535.;

    let (w, h, d) = (
        device_w as f32 / scale_factor,
        device_h as f32 / scale_factor,
        (farz - nearz),
    );

    let mut m = Matrix4::identity();

    // Scale by negating stretch caused by window
    let (sx, sy, sz) = (1.0 / w, 1.0 / h, 1.0 / d);
    m.prepend_nonuniform_scaling_mut(&Vector3::new(sx, sy, sz));

    // Translate so origin is in bottom left of window
    m.prepend_translation_mut(&Vector3::new(-w, -h, 0.0));

    m
}
