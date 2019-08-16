use glutin::dpi::{LogicalSize, PhysicalSize};
use glutin::WindowedContext;

#[derive(Debug)]
pub struct DeviceDimensions {
    pub(crate) dpi_factor: f64,
    pub(crate) logical_size: LogicalSize,
    pub(crate) physical_size: PhysicalSize,
    // TODO: positions
}

impl DeviceDimensions {
    pub fn new(dpi: f64, logical_size: LogicalSize) -> Self {
        DeviceDimensions {
            dpi_factor: dpi,
            logical_size,
            physical_size: logical_size.to_physical(dpi),
        }
    }

    /// Will fail if window is closed
    pub fn from_window(window_context: &WindowedContext<glutin::PossiblyCurrent>) -> Option<Self> {
        let dpi_factor = window_context.window().get_hidpi_factor();
        match window_context.window().get_inner_size() {
            Some(logical_size) => Some(DeviceDimensions::new(dpi_factor, logical_size)),
            // Window no longer exists
            None => None,
        }
    }

    pub fn dpi_factor(&self) -> f64 {
        self.dpi_factor
    }

    pub fn physical_size(&self) -> &PhysicalSize {
        &self.physical_size
    }
}

impl Default for DeviceDimensions {
    fn default() -> Self {
        DeviceDimensions {
            dpi_factor: 1.0,
            logical_size: LogicalSize::new(0., 0.),
            physical_size: PhysicalSize::new(0., 0.),
        }
    }
}
