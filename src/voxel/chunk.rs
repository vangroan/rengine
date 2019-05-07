use crate::voxel::{ChunkCoord, VoxelCoord, VoxelData};
use specs::{Component, DenseVecStorage};

pub const ChunkSize8: usize = 8 * 8 * 8;

pub trait VoxelChunk<D: VoxelData> {
    fn index(&self) -> &ChunkCoord;
    fn get<V: Into<VoxelCoord>>(&self, coord: V) -> &D;
    fn set<V: Into<VoxelCoord>>(&self, coord: V, data: D);
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
/// Adjacency information can be used to occlude cuboid
/// faces that are obstructed by neighouring voxels, and
/// thus don't have to be drawn.
///
/// By bookkeeping the adjacency information, the time
/// taken is sinked during updating of the chunk, freeing
/// up the iteration from performing neighbour lookups.
///
/// No deduplication or compression is applied to the
/// data.
#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct VoxelArrayChunk<D: 'static + VoxelData + Sync + Send> {
    /// Global position of the bottom, left,
    /// back voxel. Coordinate (0, 0, 0) in
    /// the chunk's local space.
    coord: ChunkCoord,

    /// Voxel data packed with adjacency map,
    /// describing whether neighbours are occupied
    /// or empty.
    data: [(VoxelAdjacency, D); ChunkSize8],
}

impl<D> VoxelArrayChunk<D>
where
    D: 'static + VoxelData + Sync + Send + Default + Copy,
{
    pub fn new<V>(coord: V) -> Self
    where
        V: Into<ChunkCoord>,
    {
        VoxelArrayChunk {
            coord: coord.into(),
            data: [Default::default(); ChunkSize8],
        }
    }
}

impl<D> VoxelChunk<D> for VoxelArrayChunk<D>
where
    D: 'static + VoxelData + Sync + Send,
{
    fn index(&self) -> &ChunkCoord {
        &self.coord
    }

    fn get<V>(&self, coord: V) -> &D
    where
        V: Into<VoxelCoord>,
    {
        unimplemented!()
    }

    fn set<V>(&self, coord: V, data: D)
    where
        V: Into<VoxelCoord>,
    {
        unimplemented!()
    }
}

type VoxelAdjacency = u32;
