use std::fmt;
use std::ops::{Add, Sub};

/// Position of a voxel in the grid.
///
/// Float positions can implicitly be converted
/// to a coordinate, with rounding handled correctly.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub struct VoxelCoord {
    pub i: i32,
    pub j: i32,
    pub k: i32,
}

impl VoxelCoord {
    pub fn new(i: i32, j: i32, k: i32) -> Self {
        VoxelCoord { i, j, k }
    }

    /// Tests for diagonality on the i, j plane
    pub fn is_diagonal_k(&self, rhs: &VoxelCoord) -> bool {
        let x = rhs.i - self.i;
        let y = rhs.j - self.j;
        x != 0 && y != 0
    }
}

impl Default for VoxelCoord {
    fn default() -> Self {
        VoxelCoord::new(0, 0, 0)
    }
}

impl fmt::Display for VoxelCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.i, self.j, self.k)
    }
}

impl Add for VoxelCoord {
    type Output = VoxelCoord;

    fn add(self, rhs: Self) -> Self::Output {
        VoxelCoord {
            i: self.i + rhs.i,
            j: self.j + rhs.j,
            k: self.k + rhs.k,
        }
    }
}

impl Add<&VoxelCoord> for &VoxelCoord {
    type Output = VoxelCoord;

    fn add(self, rhs: &VoxelCoord) -> Self::Output {
        VoxelCoord {
            i: self.i + rhs.i,
            j: self.j + rhs.j,
            k: self.k + rhs.k,
        }
    }
}

impl Sub for VoxelCoord {
    type Output = VoxelCoord;

    fn sub(self, rhs: Self) -> Self::Output {
        VoxelCoord {
            i: self.i - rhs.i,
            j: self.j - rhs.j,
            k: self.k - rhs.k,
        }
    }
}

impl Sub<&VoxelCoord> for VoxelCoord {
    type Output = VoxelCoord;

    fn sub(self, rhs: &VoxelCoord) -> Self::Output {
        VoxelCoord {
            i: self.i - rhs.i,
            j: self.j - rhs.j,
            k: self.k - rhs.k,
        }
    }
}

impl Into<nalgebra::Point3<i32>> for VoxelCoord {
    fn into(self) -> nalgebra::Point3<i32> {
        nalgebra::Point3::new(self.i, self.j, self.k)
    }
}

impl Into<(i32, i32, i32)> for VoxelCoord {
    fn into(self) -> (i32, i32, i32) {
        (self.i, self.j, self.k)
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

impl From<&[i32; 3]> for VoxelCoord {
    fn from(val: &[i32; 3]) -> VoxelCoord {
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

/// Identity of a chunk in the grid.
///
/// Chunk space normalises a single chunk
/// to size (1.0, 1.0, 1.0).
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde-serialize", derive(Serialize, Deserialize))]
pub struct ChunkCoord {
    pub i: i32,
    pub j: i32,
    pub k: i32,
}

impl ChunkCoord {
    pub fn new(i: i32, j: i32, k: i32) -> Self {
        ChunkCoord { i, j, k }
    }
}

impl Default for ChunkCoord {
    fn default() -> Self {
        ChunkCoord::new(0, 0, 0)
    }
}

impl fmt::Display for ChunkCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.i, self.j, self.k)
    }
}

impl From<[i32; 3]> for ChunkCoord {
    fn from(val: [i32; 3]) -> ChunkCoord {
        ChunkCoord {
            i: val[0],
            j: val[1],
            k: val[2],
        }
    }
}

impl From<[f32; 3]> for ChunkCoord {
    fn from(val: [f32; 3]) -> ChunkCoord {
        ChunkCoord {
            i: val[0].floor() as i32,
            j: val[1].floor() as i32,
            k: val[2].floor() as i32,
        }
    }
}

impl From<(i32, i32, i32)> for ChunkCoord {
    fn from(val: (i32, i32, i32)) -> ChunkCoord {
        ChunkCoord {
            i: val.0,
            j: val.1,
            k: val.2,
        }
    }
}

impl From<(f32, f32, f32)> for ChunkCoord {
    fn from(val: (f32, f32, f32)) -> ChunkCoord {
        ChunkCoord {
            i: val.0.floor() as i32,
            j: val.1.floor() as i32,
            k: val.2.floor() as i32,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_voxel_coord_arithmetic() {
        assert_eq!(
            VoxelCoord::new(1, 2, 3),
            VoxelCoord::new(0, 1, 2) + VoxelCoord::new(1, 1, 1),
            "Adding voxel coordinate by value failed"
        );
        assert_eq!(
            VoxelCoord::new(1, 2, 3),
            VoxelCoord::new(0, 1, 2) + VoxelCoord::new(1, 1, 1),
            "Adding volel coordinate by reference failed"
        );
    }
}
