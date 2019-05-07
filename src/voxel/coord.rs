use std::fmt;

/// Position of a voxel in the grid.
///
/// Float positions can implicitly be converted
/// to a coordinate, with rounding handled correctly.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VoxelCoord {
    pub i: i32,
    pub j: i32,
    pub k: i32,
}

impl VoxelCoord {
    pub fn new(i: i32, j: i32, k: i32) -> Self {
        VoxelCoord { i, j, k }
    }
}

impl fmt::Display for VoxelCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.i, self.j, self.k)
    }
}

impl Into<nalgebra::Point3<i32>> for VoxelCoord {
    fn into(self) -> nalgebra::Point3<i32> {
        nalgebra::Point3::new(self.i, self.j, self.k)
    }
}

impl From<[i32; 3]> for VoxelCoord {
    fn from(val: [i32; 3]) -> VoxelCoord {
        VoxelCoord {
            i: val[0],
            j: val[1],
            k: val[2],
        }
    }
}

impl From<[f32; 3]> for VoxelCoord {
    fn from(val: [f32; 3]) -> VoxelCoord {
        VoxelCoord {
            i: val[0].floor() as i32,
            j: val[1].floor() as i32,
            k: val[2].floor() as i32,
        }
    }
}

impl From<(i32, i32, i32)> for VoxelCoord {
    fn from(val: (i32, i32, i32)) -> VoxelCoord {
        VoxelCoord {
            i: val.0,
            j: val.1,
            k: val.2,
        }
    }
}

impl From<(f32, f32, f32)> for VoxelCoord {
    fn from(val: (f32, f32, f32)) -> VoxelCoord {
        VoxelCoord {
            i: val.0.floor() as i32,
            j: val.1.floor() as i32,
            k: val.2.floor() as i32,
        }
    }
}