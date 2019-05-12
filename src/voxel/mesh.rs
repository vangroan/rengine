use crate::comp::{GlTexture, MeshBuilder};
use crate::voxel::{VoxelChunk, VoxelData};

/// Mesh generator for voxel chunks.
pub trait VoxelMeshGen {
    /// The resulting mesh will be staged inside the provided
    /// mesh builder.
    fn generate<D: VoxelData, C: VoxelChunk<D>>(
        &self,
        chunk: &C,
        mesh_builder: MeshBuilder,
    ) -> MeshBuilder;
}

/// Simplest mesh generator, where each voxel is
/// a pseudocube. No occlusion on faces will be
/// performed.
pub struct VoxelBoxGen {
    texture: GlTexture,
}

impl VoxelMeshGen for VoxelBoxGen {
    fn generate<D, C>(&self, chunk: &C, mut builder: MeshBuilder) -> MeshBuilder
    where
        D: VoxelData,
        C: VoxelChunk<D>,
    {
        let dim = chunk.dim();

        for x in 0..dim {
            for y in 0..dim {
                for z in 0..dim {
                    let tex_rects = [
                        self.texture.source_rect(),
                        self.texture.source_rect(),
                        self.texture.source_rect(),
                        self.texture.source_rect(),
                        self.texture.source_rect(),
                        self.texture.source_rect(),
                    ];

                    builder = builder.pseudocube(
                        [x as f32, y as f32, z as f32],
                        [1.0, 1.0, 1.0],
                        tex_rects,
                    );
                }
            }
        }

        builder
    }
}

/// Mesh generator implementation
/// that does nothing.
///
/// Used for testing.
pub struct NoOpVoxelMeshGen;

impl VoxelMeshGen for NoOpVoxelMeshGen {
    fn generate<D, C>(&self, _chunk: &C, mut _builder: MeshBuilder) -> MeshBuilder
    where
        D: VoxelData,
        C: VoxelChunk<D>,
    {
        // Do Nothing
        _builder
    }
}
