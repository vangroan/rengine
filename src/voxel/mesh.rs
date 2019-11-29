use crate::comp::{GlTexture, MeshBuilder, TexRect};
use crate::voxel::{wiggle, VoxelChunk, VoxelData};

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

// =============================================================================
// Voxel Box Mesh Generator

/// Simplest mesh generator, where each voxel is
/// a pseudocube. No occlusion on faces will be
/// performed.
pub struct VoxelBoxGen {
    /// TODO: Do we need texture here?
    _texture: GlTexture,

    /// Texture rectangles to be used for each voxel cuboid
    tex_rects: [TexRect; 6],
}

impl VoxelBoxGen {
    pub fn new(texture: GlTexture, tex_rects: [TexRect; 6]) -> Self {
        VoxelBoxGen {
            _texture: texture,
            tex_rects,
        }
    }
}

impl VoxelMeshGen for VoxelBoxGen {
    fn generate<D, C>(&self, chunk: &C, mut builder: MeshBuilder) -> MeshBuilder
    where
        D: VoxelData,
        C: VoxelChunk<D>,
    {
        let dim = chunk.dim() as i32;

        for x in 0..dim {
            for y in 0..dim {
                for z in 0..dim {
                    let occupied = chunk
                        .get_local([x, y, z])
                        .map(|data| data.occupied())
                        .unwrap_or(false);

                    if occupied {
                        builder = builder.pseudocube(
                            [x as f32, y as f32, z as f32],
                            [1.0, 1.0, 1.0],
                            self.tex_rects.clone(),
                        );
                    }
                }
            }
        }

        builder
    }
}

// =============================================================================
// Deformed Voxel Mesh Generation

/// Deforms the corner points of each voxel
/// to visually break up the grid.
pub struct DeformedBoxGen {
    /// Amount to deform points.
    force: f32,

    /// Texture rectangles to be used for each voxel cuboid
    tex_rects: [TexRect; 6],
}

impl DeformedBoxGen {
    pub fn new(force: f32, tex_rects: [TexRect; 6]) -> Self {
        DeformedBoxGen { force, tex_rects }
    }
}

impl VoxelMeshGen for DeformedBoxGen {
    fn generate<D, C>(&self, chunk: &C, mut builder: MeshBuilder) -> MeshBuilder
    where
        D: VoxelData,
        C: VoxelChunk<D>,
    {
        let dim = chunk.dim() as i32;
        let o = chunk.voxel_offset();
        let force = self.force;

        for x in 0..dim {
            for y in 0..dim {
                for z in 0..dim {
                    let occupied = chunk
                        .get_local([x, y, z])
                        .map(|data| data.occupied())
                        .unwrap_or(false);
                    let [w0, w1, w2, w3, w4, w5, w6, w7]: [glm::Vec3; 8] = [
                        wiggle(o.i + x, o.j + y, o.k + z).into(),             // p0
                        wiggle(o.i + x, o.j + y, o.k + z + 1).into(),         // p1
                        wiggle(o.i + x, o.j + y + 1, o.k + z).into(),         // p2
                        wiggle(o.i + x, o.j + y + 1, o.k + z + 1).into(),     // p3
                        wiggle(o.i + x + 1, o.j + y, o.k + z).into(),         // p4
                        wiggle(o.i + x + 1, o.j + y, o.k + z + 1).into(),     // p5
                        wiggle(o.i + x + 1, o.j + y + 1, o.k + z).into(),     // p6
                        wiggle(o.i + x + 1, o.j + y + 1, o.k + z + 1).into(), // p7
                    ];
                    let pos = glm::vec3(x as f32, y as f32, z as f32);
                    if occupied {
                        builder = builder.pseudocube_points(
                            [
                                pos + glm::vec3(0.0, 0.0, 0.0)
                                    + (w0 - glm::vec3(0.5, 0.5, 0.5)) * force, // p0
                                pos + glm::vec3(0.0, 0.0, 1.0)
                                    + (w1 - glm::vec3(0.5, 0.5, 0.5)) * force, // p1
                                pos + glm::vec3(0.0, 1.0, 0.0)
                                    + (w2 - glm::vec3(0.5, 0.5, 0.5)) * force, // p2
                                pos + glm::vec3(0.0, 1.0, 1.0)
                                    + (w3 - glm::vec3(0.5, 0.5, 0.5)) * force, // p3
                                pos + glm::vec3(1.0, 0.0, 0.0)
                                    + (w4 - glm::vec3(0.5, 0.5, 0.5)) * force, // p4
                                pos + glm::vec3(1.0, 0.0, 1.0)
                                    + (w5 - glm::vec3(0.5, 0.5, 0.5)) * force, // p5
                                pos + glm::vec3(1.0, 1.0, 0.0)
                                    + (w6 - glm::vec3(0.5, 0.5, 0.5)) * force, // p6
                                pos + glm::vec3(1.0, 1.0, 1.0)
                                    + (w7 - glm::vec3(0.5, 0.5, 0.5)) * force, // p7
                            ],
                            self.tex_rects.clone(),
                        );
                    }
                }
            }
        }

        builder
    }
}

// =============================================================================
// No-Operation Voxel Mesh Generation

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
