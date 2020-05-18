use glutin::dpi::LogicalSize;
use nalgebra::Point2;
use specs::{Component, DenseVecStorage};

/// Axis-aligned bounding box in logical pixel size.
#[derive(Component, Clone)]
#[storage(DenseVecStorage)]
pub struct WidgetBounds {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl WidgetBounds {
    pub fn new(width: f32, height: f32) -> Self {
        WidgetBounds { width, height }
    }

    pub fn from_size(size: LogicalSize) -> Self {
        WidgetBounds {
            width: size.width as f32,
            height: size.height as f32,
        }
    }

    #[inline]
    pub fn set_size<V>(&mut self, size: V)
    where
        V: Into<[f32; 2]>,
    {
        let [width, height] = size.into();
        self.width = width;
        self.height = height;
    }

    #[inline]
    pub fn size(&self) -> [f32; 2] {
        [self.width, self.height]
    }

    /// Returns whether the given point is within the local
    /// bounds, in logical pixels.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::gui::WidgetBounds;
    /// use nalgebra::Point2;
    ///
    /// let aabb = WidgetBounds::new(120.0, 70.0);
    /// assert!(aabb.intersect_point([50.0, 50.0]));
    /// assert!(!aabb.intersect_point([400.0, -200.0]));
    /// assert!(aabb.intersect_point(Point2::new(50.0, 50.0)));
    /// ```
    pub fn intersect_point<V>(&self, point: V) -> bool
    where
        V: Into<Point2<f32>>,
    {
        let p = point.into();
        p.x >= 0.0 && p.y >= 0.0 && p.x <= self.width && p.y <= self.height
    }
}
