use crate::voxel::VoxelCoord;
use specs::{Component, DenseVecStorage};

pub const ChunkSize8: usize = 8 * 8 * 8;

pub trait VoxelChunk<D: VoxelData> {}

pub trait VoxelData {
    /// Indicates whether the voxel
    /// is considered occupied, or empty.
    fn occupied(&self) -> bool;
}

/// Implementation of `VoxelChunk` that naively keeps
/// data in an array.
///
/// Each voxel record has an adjacency mapping that
/// indicates whether its neighbours are occupied or
/// empty. Occupancy from neighbouring chunks is not
/// automatically controlled, and must be enforced by
/// an upper container that has knoweldge of chunks.
///
/// No deduplication or compression is applied to the
/// data.
#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct VoxelArrayChunk<D: 'static + VoxelData + Sync + Send> {
    /// Global position of the bottom, left,
    /// back voxel. Coordinate (0, 0, 0) in
    /// the chunk's local space.
    coord: VoxelCoord,

    /// Voxel data packed with adjacency map,
    /// describing whether neighbours are occupied
    /// or empty.
    data: [(VoxelAdjacency, D); ChunkSize8],
}

type VoxelAdjacency = u32;
