/// Inexact equality comparison
/// for floating point numbers.
#[macro_export]
macro_rules! inexact_eq {
    ($lhs:expr, $rhs:expr) => {
        (f64::from($lhs) - f64::from($rhs)).abs() < ::std::f64::EPSILON
    };
}