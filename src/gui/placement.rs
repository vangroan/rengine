use nalgebra::{Matrix4, Vector2, Vector3};
use specs::{Component, DenseVecStorage};

/// Represents a relative position within a View.
///
/// To support different sized Windows and Screens, a Placement
/// can be used in a Node of the GUI graph to offset its own
/// Transform, and thus its children, by a relative distance.
///
/// During a layout pass, the GUI View and Widget's Transform are
/// used to calculate a position, which is then set as the Transform's
/// position.
///
/// The distance is a normalised Vector. A coordinate of (0.0, 0.0) is
/// the top left of the View, while (1.0, 1.0) is the bottom right.
#[derive(Debug, Component)]
#[storage(DenseVecStorage)]
pub struct Placement {
    offset: Vector2<f32>,
}

impl Placement {
    pub fn new(x: f32, y: f32) -> Self {
        Placement::from_vector(Vector2::new(x, y))
    }

    pub fn from_vector<V>(offset: V) -> Self
    where
        V: Into<Vector2<f32>>,
    {
        Placement {
            offset: offset.into(),
        }
    }

    pub fn zero() -> Self {
        Placement::new(0.0, 0.0)
    }

    #[inline]
    pub fn offset(&self) -> &Vector2<f32> {
        &self.offset
    }

    #[inline]
    pub fn set_offset<V>(&mut self, offset: V)
    where
        V: Into<Vector2<f32>>,
    {
        self.offset = offset.into();
    }

    /// Creates a model matrix from the placement's offset vector.
    ///
    /// # Example
    ///
    /// ```
    /// use rengine::gui::Placement;
    /// use nalgebra::Point3;
    ///
    /// let p = Placement::new(0.5, 0.5);
    /// let m = p.matrix();
    ///
    /// let transformed_point = m.transform_point(&Point3::new(0.0, 0.0, 0.0));
    /// assert_eq!(transformed_point, Point3::new(0.5, 0.5, 0.0));
    /// ```
    pub fn matrix(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&Vector3::<f32>::new(self.offset.x, self.offset.y, 0.0))
    }
}

impl Default for Placement {
    fn default() -> Self {
        Placement {
            offset: Vector2::new(0.0, 0.0),
        }
    }
}
