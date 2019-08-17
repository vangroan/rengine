use crate::voxel::VoxelCoord;
use nalgebra::{Point3, Unit, Vector3};
use std::f32::MAX;
use std::iter::Iterator;

#[derive(Debug, PartialEq)]
pub struct VoxelRayInfo {
    /// Length traveled along ray.
    t: f32,

    /// Point where ray entered voxel.
    intersect: Point3<f32>,

    /// Voxel that has been intersected.
    voxel: VoxelCoord,
}

impl VoxelRayInfo {
    pub fn voxel_coord(&self) -> &VoxelCoord {
        &self.voxel
    }
}

// https://lodev.org/cgtutor/raycasting.html
#[allow(dead_code)]
pub fn voxel_raycast(
    origin: Point3<f32>,
    direction: Unit<Vector3<f32>>,
    steps: u32,
) -> VoxelRaycast {
    // Initial voxel coordinate.
    //
    // Implicitly the origin is intersecting the
    // starting voxel.
    let (x, y, z) = (
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

    // Determine which direction we are stepping.
    //
    // Calculate initial lengths from origin
    // to first crossing of boundries.
    let (step_x, t_x) = if direction.x > 0.0 {
        (1, (origin.x.ceil() - origin.x).abs() * delta_x)
    } else {
        (-1, (origin.x.floor() - origin.x).abs() * delta_x)
    };

    let (step_y, t_y) = if direction.y > 0.0 {
        (1, (origin.y.ceil() - origin.y).abs() * delta_y)
    } else {
        (-1, (origin.y.floor() - origin.y).abs() * delta_y)
    };

    let (step_z, t_z) = if direction.z > 0.0 {
        (1, (origin.z.ceil() - origin.z).abs() * delta_z)
    } else {
        (-1, (origin.z.floor() - origin.z).abs() * delta_z)
    };

    VoxelRaycast {
        origin,
        direction,
        max_steps: steps,
        delta: [delta_x, delta_y, delta_z],
        step: [step_x, step_y, step_z],
        voxel: [x, y, z],
        cursor: 0,
        t: [t_x, t_y, t_z],
    }
}

pub struct VoxelRaycast {
    /// Position where ray starts
    origin: Point3<f32>,

    /// Where the ray is going
    direction: Unit<Vector3<f32>>,

    /// Number of voxels to traverse
    /// before giving up.
    max_steps: u32,

    /// Length along the ray to travel
    /// to cross a voxel border along
    /// a specific axis.
    delta: [f32; 3],

    /// Direction to step voxel coordintes.
    step: [i32; 3],

    /// Current voxel being intersected.
    voxel: [i32; 3],

    /// Current step.
    cursor: u32,

    /// Total length traveled along the
    /// ray to reach a border for each
    /// of the three axes.
    t: [f32; 3],
}

impl Iterator for VoxelRaycast {
    type Item = VoxelRayInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.max_steps {
            None
        } else {
            // Takes current state of shortest axis ray, advances state for
            // next interation, then unaltered returns state.
            let voxel_info = if self.t[0] < self.t[1] {
                if self.t[0] < self.t[2] {
                    // X-axis
                    let i = VoxelRayInfo {
                        t: self.t[0],
                        intersect: self.origin + self.direction.into_inner() * self.t[0],
                        voxel: self.voxel.into(),
                    };

                    self.t[0] += self.delta[0];
                    self.voxel[0] += self.step[0];

                    i
                } else {
                    // Z-axis
                    let i = VoxelRayInfo {
                        t: self.t[2],
                        intersect: self.origin + self.direction.into_inner() * self.t[2],
                        voxel: self.voxel.into(),
                    };

                    self.t[2] += self.delta[2];
                    self.voxel[2] += self.step[2];

                    i
                }
            } else if self.t[1] < self.t[2] {
                // Y-axis
                let i = VoxelRayInfo {
                    t: self.t[1],
                    intersect: self.origin + self.direction.into_inner() * self.t[1],
                    voxel: self.voxel.into(),
                };

                self.t[1] += self.delta[1];
                self.voxel[1] += self.step[1];

                i
            } else {
                // Z-axis
                let i = VoxelRayInfo {
                    t: self.t[2],
                    intersect: self.origin + self.direction.into_inner() * self.t[2],
                    voxel: self.voxel.into(),
                };

                self.t[2] += self.delta[2];
                self.voxel[2] += self.step[2];

                i
            };

            self.cursor += 1;

            Some(voxel_info)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_cast() {
        let ray = voxel_raycast(
            [1.5, 0.5, 0.5].into(),
            Unit::new_normalize([0.5, 0.866025403, 0.0].into()),
            10,
        );
        let target = VoxelCoord::new(2, 3, 0);
        let mut found: Option<(usize, VoxelRayInfo)> = None;

        for (i, ray_info) in ray.enumerate() {
            println!("Voxel Coord: {}", ray_info.voxel_coord());
            if ray_info.voxel == target {
                found = Some((i, ray_info));
                break;
            }
        }

        assert!(found.is_some());
        let (cursor, info) = found.unwrap();
        println!("{}", info.intersect);
        assert_eq!(target, info.voxel);
        assert_eq!(4, cursor);
    }
}
