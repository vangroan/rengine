use nalgebra::{Point3, Unit, Vector3};
use std::f32::MAX;

#[derive(Debug, PartialEq, Eq)]
pub struct VoxelRayInfo;

// https://lodev.org/cgtutor/raycasting.html
#[allow(dead_code)]
fn voxel_raycast(
    origin: Point3<f32>,
    direction: Unit<Vector3<f32>>,
    steps: usize,
) -> Option<VoxelRayInfo> {
    let (mut x, mut y, mut z) = (
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    );

    // The length along the ray we need to travel to
    // cross a voxel border along a specific axis.
    //
    // When the direction is 0.0 along an axis, then
    // it is parallel, and will never cross the axis.
    let delta_x = if direction.x != 0.0 {
        (1.0 / direction.x).abs()
    } else {
        MAX
    };

    let delta_y = if direction.y != 0.0 {
        (1.0 / direction.y).abs()
    } else {
        MAX
    };

    let delta_z = if direction.z != 0.0 {
        (1.0 / direction.z).abs()
    } else {
        MAX
    };

    println!("delta ({}, {}, {})", delta_x, delta_y, delta_z);

    // Determine which direction we are stepping.
    //
    // Calculate initial lengths from origin
    // to first crossing of boundries.
    let (step_x, mut max_x) = if direction.x > 0.0 {
        (1, (origin.x.ceil() - origin.x).abs() * delta_x)
    } else {
        (-1, (origin.x.floor() - origin.x).abs() * delta_x)
    };

    let (step_y, mut max_y) = if direction.y > 0.0 {
        (1, (origin.y.ceil() - origin.y).abs() * delta_y)
    } else {
        (-1, (origin.y.floor() - origin.y).abs() * delta_y)
    };

    let (step_z, mut max_z) = if direction.z > 0.0 {
        (1, (origin.z.ceil() - origin.z).abs() * delta_z)
    } else {
        (-1, (origin.z.floor() - origin.z).abs() * delta_z)
    };

    println!("step ({}, {}, {})", step_x, step_y, step_z);
    println!("t ({}, {}, {})", max_x, max_y, max_y);

    let mut i = 0;
    while i < steps {
        println!("({}, {}, {}) t ({}, {}, {})", x, y, z, max_x, max_y, max_z);

        if max_x < max_y {
            if max_x < max_z {
                max_x += delta_x;
                x += step_x;
            } else {
                max_z += delta_z;
                z += step_z;
            }
        } else {
            if max_y < max_z {
                max_y += delta_y;
                y += step_y;
            } else {
                max_z += delta_z;
                z += step_z;
            }
        }

        i += 1;
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_cast() {
        let info = voxel_raycast(
            [1.5, 0.5, 0.1].into(),
            Unit::new_normalize([0.5, 0.866025403, 0.0].into()),
            10,
        );
        assert_eq!(None, info);
    }
}
