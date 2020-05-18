use nalgebra::Point2;
use specs::{Component, DenseVecStorage};

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct GlobalPosition(Point2<f32>);

impl GlobalPosition {
    pub fn new(x: f32, y: f32) -> Self {
        GlobalPosition(Point2::new(x, y))
    }

    #[inline]
    pub fn point(&self) -> Point2<f32> {
        self.0
    }

    #[inline]
    pub fn set_point<V>(&mut self, point: V)
    where
        V: Into<Point2<f32>>,
    {
        self.0 = point.into()
    }
}
