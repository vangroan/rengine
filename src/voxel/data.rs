pub trait VoxelData {
    /// Indicates whether the voxel
    /// is considered occupied, or empty.
    fn occupied(&self) -> bool;
}

/// Implicit convenience implementation for
/// small unsigned integers. Allows for 65535
/// voxel types. For use in simple case.
impl VoxelData for u16 {
    fn occupied(&self) -> bool {
        *self != 0
    }
}
