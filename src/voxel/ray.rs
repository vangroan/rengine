use nalgebra::{Point3, Unit, Vector3};
use std::f32::INFINITY;

#[derive(Debug, PartialEq, Eq)]
pub struct VoxelRayInfo;

// https://lodev.org/cgtutor/raycasting.html
#[allow(dead_code)]
fn voxel_raycast<P, V>(
    origin: Point3<f32>,
    direction: Unit<Vector3<f32>>,
    steps: usize,
) -> Option<VoxelRayInfo> {
    // Determine direction for steps -1 or 1
    let step_x = if direction.x > 0.0 { 1 } else { -1 };
    let step_y = if direction.y > 0.0 { 1 } else { -1 };
    let step_z = if direction.z > 0.0 { 1 } else { -1 };

    // The length along the ray we need to travel to
    // cross a voxel border along a specific axis.
    //
    // When the direction is 0.0 along an axis, then
    // it is parallel, and will never cross the axis.
    let delta_x = if direction.x != 0.0 {
        (1.0 / direction.x).abs()
    } else {
        INFINITY
    };

    let delta_y = if direction.y != 0.0 {
        (1.0 / direction.y).abs()
    } else {
        INFINITY
    };

    let delta_z = if direction.z != 0.0 {
        (1.0 / direction.z).abs()
    } else {
        INFINITY
    };

    // Calculate initial lengths from origin
    // to first crossing of boundries.
    let mix_x = if direction.x > 0.0 {

    } else {

    };

    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_cast() {
        let info = voxel_raycast(
            [0., 0., 0.].into(),
            Unit::new_normalize([0.5, 0.25, 0.0].into()),
            10,
        );
        assert_eq!(None, info);
    }
}
