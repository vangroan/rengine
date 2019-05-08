use crate::voxel::{ChunkCoord, VoxelCoord, VoxelData};
use specs::{Component, DenseVecStorage};

pub const ChunkDim8: usize = 8;
pub const ChunkSize8: usize = ChunkDim8 * ChunkDim8 * ChunkDim8;
// pub type VoxelOffsets: [VoxelCoord]

/// Given a global voxel coordinate, return
/// the chunk coordinate that contains it.
pub fn voxel_to_chunk(v: &VoxelCoord) -> ChunkCoord {
    // Integer division truncates, meaning negative
    // numbers round towards 0, so we need to do
    // a pass through floating point maths to get
    // floor behaviour.
    ChunkCoord {
        i: (v.i as f32 / ChunkDim8 as f32).floor() as i32,
        j: (v.j as f32 / ChunkDim8 as f32).floor() as i32,
        k: (v.k as f32 / ChunkDim8 as f32).floor() as i32,
    }
}

pub trait VoxelChunk<D: VoxelData> {
    fn index(&self) -> &ChunkCoord;
    fn in_bounds<V: Into<VoxelCoord>>(&self, coord: V) -> bool;
    fn get<V: Into<VoxelCoord>>(&self, coord: V) -> Option<&D>;
    fn set<V: Into<VoxelCoord>>(&mut self, coord: V, data: D);
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
    /// Unique identifier for this chunk.
    coord: ChunkCoord,

    /// Global position of the bottom, left,
    /// back voxel. Coordinate (0, 0, 0) in
    /// the chunk's local space.
    voxel_offset: VoxelCoord,

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
        let chunk_coord = coord.into();

        // Translate chunk coordinates to voxel coordinates
        let voxel_offset = VoxelCoord::new(
            chunk_coord.i * ChunkDim8 as i32,
            chunk_coord.j * ChunkDim8 as i32,
            chunk_coord.k * ChunkDim8 as i32,
        );

        VoxelArrayChunk {
            coord: chunk_coord,
            voxel_offset,
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

    fn in_bounds<V>(&self, coord: V) -> bool
    where
        V: Into<VoxelCoord>,
    {
        let VoxelCoord { i, j, k } = coord.into();
        let (i1, j1, k1) = self.voxel_offset.clone().into();
        let (i2, j2, k2) = (
            i1 + ChunkDim8 as i32,
            j1 + ChunkDim8 as i32,
            k1 + ChunkDim8 as i32,
        );

        i >= i1 && j >= j1 && k >= k1 && i < i2 && j < j2 && k < k2
    }

    fn get<V>(&self, coord: V) -> Option<&D>
    where
        V: Into<VoxelCoord>,
    {
        let voxel_coord: VoxelCoord = coord.into();

        if self.in_bounds(voxel_coord.clone()) {
            let index: usize = (voxel_coord.i
                + voxel_coord.j * ChunkDim8 as i32
                + voxel_coord.k * ChunkDim8 as i32 * ChunkDim8 as i32)
                as usize;
            self.data.get(index).map(|el| &el.1)
        } else {
            None
        }
    }

    fn set<V>(&mut self, coord: V, data: D)
    where
        V: Into<VoxelCoord>,
    {
        let voxel_coord: VoxelCoord = coord.into();
        // TODO: Set all adjacent
        if self.in_bounds(voxel_coord.clone()) {
            let index: usize = (voxel_coord.i
                + voxel_coord.j * ChunkDim8 as i32
                + voxel_coord.k * ChunkDim8 as i32 * ChunkDim8 as i32)
                as usize;
            self.data[index] = (Default::default(), data);
        }
    }
}

type VoxelAdjacency = u32;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_voxel_to_chunk() {
        let v1 = VoxelCoord::new(7, 6, 5);
        assert_eq!(ChunkCoord::new(0, 0, 0), voxel_to_chunk(&v1));

        let v2 = VoxelCoord::new(7, 8, 5);
        assert_eq!(ChunkCoord::new(0, 1, 0), voxel_to_chunk(&v2));

        let v3 = VoxelCoord::new(-7, 8, 5);
        assert_eq!(ChunkCoord::new(-1, 1, 0), voxel_to_chunk(&v3));
    }
}
