use num_traits::cast::FromPrimitive;
use num_traits::float::{Float, FloatConst};

pub struct Deg<N: Float>(pub N);

impl<N> Deg<N>
where
    N: Float + FromPrimitive + FloatConst,
{
    #[inline]
    pub fn as_radians(&self) -> N {
        let d: N = N::from_f64(180.).unwrap();
        let pi = N::PI();
        self.0 * (pi / d)
    }

    #[inline]
    pub fn as_degrees(&self) -> N {
        self.0
    }
}

pub struct Rad<N: Float>(pub N);

impl<N> Rad<N>
where
    N: Float + FromPrimitive + FloatConst,
{
    #[inline]
    pub fn as_radians(&self) -> N {
        self.0
    }

    #[inline]
    pub fn as_degrees(&self) -> N {
        let d: N = N::from_f64(180.).unwrap();
        let pi = N::PI();
        self.0 * (d / pi)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_degrees() {
        let deg_45 = Deg(45.);
        assert_eq!(::std::f32::consts::PI / 4., deg_45.as_radians());
    }

    #[test]
    fn test_radians() {
        let pi = ::std::f32::consts::PI;
        let rad_45 = Rad(pi / 4.);
        assert_eq!(45., rad_45.as_degrees());
    }
}
