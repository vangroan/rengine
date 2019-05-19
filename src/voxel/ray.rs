use nalgebra::{Point3, Vector3};

#[derive(Debug, PartialEq, Eq)]
pub struct VoxelRayInfo;

#[allow(dead_code)]
fn voxel_raycast<P, V, F>(
    origin: P,
    direction: V,
    steps: usize,
    predicate: F,
) -> Option<VoxelRayInfo>
where
    P: Into<Point3<f32>>,
    V: Into<Vector3<f32>>,
    F: Fn(i32, i32, i32) -> bool,
{
    let origin_p: Point3<f32> = origin.into();
    let dir_v: Vector3<f32> = direction.into();

    // Determine direction for steps -1 or 1
    // Round initial step to closest voxel boundry
    let (x_delta, mut x_max) = if dir_v.x >= 0.0 {
        (1.0, origin_p.x.ceil())
    } else {
        (-1.0, origin_p.x.floor())
    };

    let (y_delta, mut y_max) = if dir_v.y >= 0.0 {
        (1.0, origin_p.y.ceil())
    } else {
        (-1.0, origin_p.y.floor())
    };

    let (z_delta, mut z_max) = if dir_v.z >= 0.0 {
        (1.0, origin_p.z.ceil())
    } else {
        (-1.0, origin_p.z.floor())
    };

    let mut i = 0;
    while i < steps {
        let x = x_max as i32;
        let y = y_max as i32;
        let z = z_max as i32;

        if predicate(x, y, z) {
            return Some(VoxelRayInfo);
        } else {
            if x < y {
                if x < z {
                    x_max += x_delta;
                } else {
                    z_max += z_delta;
                }
            } else {
                if y < z {
                    y_max += y_delta;
                } else {
                    z_max += z_delta;
                }
            }

            i += 1;
        }
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_cast() {
        let info = voxel_raycast([0., 0., 0.], [0.5, 0.25, 0.0], 10, |x, y, z| false);
        assert_eq!(None, info);
    }
}
