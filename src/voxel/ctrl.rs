use crate::voxel::{VoxelChunk, VoxelCoord, VoxelData};
use specs::{Component, Entity, WriteStorage};
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;

/// Global control of multiple chunks, to enforce
/// rules accross sibling chunks.
///
/// Chunks are expected to be associated with entities,
/// and thus kept in component storage. A mapping of
/// chunk coordinates to entity identifiers are kept
/// in a lookup.
pub struct ChunkControl<D: VoxelData, C: VoxelChunk<D>> {
    /// Mapping of chunks to entities
    /// linked to `VoxelChunk` instances.
    chunks: HashMap<VoxelCoord, Entity>,
    cmds: VecDeque<LazyCommand<D>>,
    _marker: PhantomData<(D, C)>,
}

impl<D, C> ChunkControl<D, C>
where
    D: VoxelData,
    C: VoxelChunk<D>,
{
    pub fn new() -> Self {
        Default::default()
    }

    /// Queues an update to voxel data at the given
    /// position, potentially for multiple chunks.
    pub fn lazy_update(&mut self, coord: VoxelCoord, data: D) {
        unimplemented!()
    }

    /// Applies queued updates to chunks, and regenerates
    /// the chunk's mesh.
    pub fn maintain(&self, chunks: &WriteStorage<'_, C>)
    where
        C: Component,
    {
        unimplemented!()
    }
}

impl<D, C> Default for ChunkControl<D, C>
where
    D: VoxelData,
    C: VoxelChunk<D>,
{
    fn default() -> Self {
        ChunkControl {
            chunks: HashMap::new(),
            cmds: VecDeque::new(),
            _marker: PhantomData,
        }
    }
}

enum LazyCommand<D: VoxelData> {
    UpdateData(VoxelCoord, D),
}
