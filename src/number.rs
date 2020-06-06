//! Utilities for working with generic numbers.
use num_traits::Float;
use std::cmp::Ordering;

/// Wrapper for a floating point number that's not NaN.
///
/// ```
/// use rengine::number::NonNan;
///
/// assert_eq!(NonNan::new(3.0).unwrap().into_inner(), 3.0);
/// assert_eq!(NonNan::new(::std::f64::NAN), None);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct NonNan<F: Float>(F);

impl<F> NonNan<F>
where
    F: Float,
{
    #[inline]
    pub fn new(val: F) -> Option<NonNan<F>> {
        if val.is_nan() {
            None
        } else {
            Some(NonNan(val))
        }
    }

    #[inline]
    pub fn into_inner(self) -> F {
        self.0
    }
}

impl<F> Eq for NonNan<F> where F: Float {}

impl<F> Ord for NonNan<F>
where
    F: Float,
{
    #[inline]
    fn cmp(&self, rhs: &NonNan<F>) -> Ordering {
        self.0.partial_cmp(&rhs.0).unwrap()
    }
}
