use crate::voxel::{voxel_to_chunk, ChunkCoord, VoxelChunk, VoxelCoord, VoxelData};
use specs::{Component, Entity, WriteStorage};
use std::collections::HashMap;
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
    chunks: HashMap<ChunkCoord, Entity>,
    cmds: Vec<LazyCommand<D>>,
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

    /// Adds a reverse mapping from the chunk's coordinate
    /// to the `Entity`.
    pub fn add_chunk(&mut self, entity: Entity, chunk: &C) {
        self.chunks.insert(chunk.index().clone(), entity);
    }

    /// Queues an update to voxel data at the given
    /// position, potentially for multiple chunks.
    pub fn lazy_update<V>(&mut self, coord: V, data: D)
    where
        V: Into<VoxelCoord>,
    {
        self.cmds.push(LazyCommand::UpdateData(coord.into(), data));
    }

    /// Returns number of commands waiting in the queue.
    pub fn cmd_len(&self) -> usize {
        self.cmds.len()
    }

    /// Applies queued updates to chunks, and regenerates
    /// the chunk's mesh.
    pub fn maintain(&mut self, chunks: &mut WriteStorage<'_, C>)
    where
        C: Component,
    {
        use LazyCommand::*;

        for cmd in self.cmds.drain(..).into_iter() {
            match cmd {
                UpdateData(voxel_coord, voxel_data) => {
                    // Convert voxel coordinate to chunk coordinate
                    let chunk_coord = voxel_to_chunk(&voxel_coord);

                    // Retrieve chunk entity
                    if let Some(entity) = self.chunks.get(&chunk_coord) {
                        // Retireve chunk component
                        if let Some(chunk) = chunks.get_mut(*entity) {
                            // Update chunk data
                            chunk.set(voxel_coord, voxel_data);
                        }
                    }
                }
            }
        }
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
            cmds: Vec::new(),
            _marker: PhantomData,
        }
    }
}

enum LazyCommand<D: VoxelData> {
    UpdateData(VoxelCoord, D),
}
