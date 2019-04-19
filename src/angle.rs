use num_traits::cast::FromPrimitive;
use num_traits::float::{Float, FloatConst};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
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

impl<N> Into<Rad<N>> for Deg<N>
where
    N: Float + FromPrimitive + FloatConst,
{
    fn into(self) -> Rad<N> {
        Rad(self.as_radians())
    }
}

impl<N> fmt::Display for Deg<N>
where
    N: Float + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({})", self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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

impl<N> Into<Deg<N>> for Rad<N>
where
    N: Float + FromPrimitive + FloatConst,
{
    fn into(self) -> Deg<N> {
        Deg(self.as_degrees())
    }
}

impl<N> fmt::Display for Rad<N>
where
    N: Float + fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({})", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_degrees() {
        let deg_45 = Deg(45.);
        let rad_45 = ::std::f32::consts::PI / 4.;
        assert_eq!(rad_45, deg_45.as_radians());
        assert_eq!(Rad(rad_45), deg_45.into());
    }

    #[test]
    fn test_radians() {
        let pi = ::std::f32::consts::PI;
        let deg_45 = 45.;
        let rad_45 = Rad(pi / 4.);
        assert_eq!(deg_45, rad_45.as_degrees());
        assert_eq!(Deg(deg_45), rad_45.into());
    }
}
