use crate::comp::{Mesh, MeshBuilder};
use crate::voxel::{voxel_to_chunk, ChunkCoord, VoxelChunk, VoxelCoord, VoxelData, VoxelMeshGen};
use specs::{Component, Entity, Read, System, Write, WriteStorage};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

/// Global control of multiple chunks, to enforce
/// rules accross sibling chunks.
pub struct ChunkControl<D: VoxelData, C: VoxelChunk<D>> {
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
}

impl<D, C> Default for ChunkControl<D, C>
where
    D: VoxelData,
    C: VoxelChunk<D>,
{
    fn default() -> Self {
        ChunkControl {
            cmds: Vec::new(),
            _marker: PhantomData,
        }
    }
}

enum LazyCommand<D: VoxelData> {
    UpdateData(VoxelCoord, D),
}

/// Mapping of Entity IDs to Chunk components.
///
/// Chunks are expected to be associated with their own
/// entities, and thus kept in component storage. This
/// mapping allows for a lookup of chunk coordinates
/// to an entity identity.
#[derive(Default)]
pub struct ChunkMapping(HashMap<ChunkCoord, Entity>);

impl ChunkMapping {
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a reverse mapping from the chunk's coordinate
    /// to the `Entity`.
    pub fn add_chunk<V>(&mut self, entity: Entity, chunk_coord: V)
    where
        V: Into<ChunkCoord>,
    {
        self.0.insert(chunk_coord.into(), entity);
    }
}

/// Applies queued updates to chunks, and regenerates
/// the chunk's mesh.
pub struct ChunkUpkeepSystem<D: VoxelData, C: VoxelChunk<D>, G: VoxelMeshGen> {
    /// Chunks touched by update, that needs updating.
    ///
    /// Kept in struct to avoid constnt allocation.
    dirty: HashSet<ChunkCoord>,
    _marker: PhantomData<(D, C, G)>,
}

impl<D, C, G> ChunkUpkeepSystem<D, C, G>
where
    D: VoxelData,
    C: VoxelChunk<D>,
    G: 'static + VoxelMeshGen + Send + Sync,
{
    pub fn new() -> Self {
        ChunkUpkeepSystem {
            dirty: HashSet::new(),
            _marker: PhantomData,
        }
    }
}

impl<'a, D, C, G> System<'a> for ChunkUpkeepSystem<D, C, G>
where
    D: 'static + VoxelData + Send + Sync,
    C: 'static + VoxelChunk<D> + Component + Send + Sync,
    G: 'static + VoxelMeshGen + Send + Sync,
{
    type SystemData = (
        Write<'a, ChunkControl<D, C>>,
        Write<'a, ChunkMapping>,
        WriteStorage<'a, C>,
        WriteStorage<'a, Mesh>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use LazyCommand::*;
        let (mut chunk_ctrl, chunk_map, mut chunks, mut _mesh_gen) = data;

        for cmd in chunk_ctrl.cmds.drain(..).into_iter() {
            match cmd {
                UpdateData(voxel_coord, voxel_data) => {
                    // Convert voxel coordinate to chunk coordinate
                    let chunk_coord = voxel_to_chunk(&voxel_coord);

                    // Retrieve chunk entity
                    if let Some(entity) = chunk_map.0.get(&chunk_coord) {
                        // Retireve chunk component
                        if let Some(chunk) = chunks.get_mut(*entity) {
                            // Update chunk data
                            chunk.set(voxel_coord, voxel_data);
                            self.dirty.insert(chunk_coord.clone());
                        }
                    }
                }
            }
        }

        if !self.dirty.is_empty() {
            for chunk_coord in self.dirty.iter() {
                // Retrieve chunk entity
                if let Some(entity) = chunk_map.0.get(&chunk_coord) {
                    // Retireve chunk component
                    if let Some(chunk) = chunks.get_mut(*entity) {

                        // mesh_gen.generate(chunk, MeshBuilder::new())
                        //     .build();
                    }
                }
            }

            self.dirty.clear();
        }
    }
}
