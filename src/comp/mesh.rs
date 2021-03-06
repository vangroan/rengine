use crate::colors::{Color, WHITE};
use crate::comp::TexRect;
use crate::gfx_types::{Transform, Vertex};
use crate::graphics::GraphicContext;
use gfx::handle::Buffer;
use gfx::traits::FactoryExt;
use gfx::Slice;
use specs::prelude::*;
use std::collections::VecDeque;

// http://ilkinulas.github.io/development/unity/2016/05/06/uv-mapping.html

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Mesh {
    pub(crate) vbuf: Buffer<gfx_device::Resources, Vertex>,
    pub(crate) slice: Slice<gfx_device::Resources>,
    pub(crate) transbuf: Buffer<gfx_device::Resources, Transform>,
}

pub struct MeshBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Default for MeshBuilder {
    fn default() -> Self {
        MeshBuilder {
            vertices: vec![],
            indices: vec![],
        }
    }
}

impl MeshBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    /// New vertices will be inserted starting here
    #[inline]
    fn next_index(&self) -> u16 {
        self.vertices.len() as u16
    }

    /// Create a pseudocube from the given points, representing the corners.
    ///
    /// | Point | x | y | z |
    /// |:-----:|:-:|:-:|:-:|
    /// | p0    | 0 | 0 | 0 |
    /// | p1    | 0 | 0 | 1 |
    /// | p2    | 0 | 1 | 0 |
    /// | p3    | 0 | 1 | 1 |
    /// | p4    | 1 | 0 | 0 |
    /// | p5    | 1 | 0 | 1 |
    /// | p6    | 1 | 1 | 0 |
    /// | p7    | 1 | 1 | 1 |
    pub fn pseudocube_points<V>(mut self, points: [V; 8], texture_rects: [TexRect; 6]) -> Self
    where
        V: Into<glm::Vec3>,
    {
        let [v0, v1, v2, v3, v4, v5, v6, v7] = points;
        let [p0, p1, p2, p3, p4, p5, p6, p7]: [[f32; 3]; 8] = [
            v0.into().into(),
            v1.into().into(),
            v2.into().into(),
            v3.into().into(),
            v4.into().into(),
            v5.into().into(),
            v6.into().into(),
            v7.into().into(),
        ];
        let [back_tex, front_tex, left_tex, right_tex, bottom_tex, top_tex] = texture_rects;
        let index = self.next_index();

        // Back Quad
        let normal = glm::vec3(0., 0., -1.).into();
        self.vertices.extend(&[
            Vertex {
                pos: p4,
                uv: [back_tex.x(), back_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p0,
                uv: [back_tex.w(), back_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p2,
                uv: [back_tex.w(), back_tex.y()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p6,
                uv: [back_tex.x(), back_tex.y()],
                normal,
                color: WHITE,
            },
        ]);

        // triangle 1
        self.indices.extend(&[index, index + 1, index + 2]);

        // triangle 2
        self.indices.extend(&[index, index + 2, index + 3]);

        // Front Quad
        let normal = glm::vec3(0., 0., 1.).into();
        self.vertices.extend(&[
            Vertex {
                pos: p1,
                uv: [front_tex.x(), front_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p5,
                uv: [front_tex.w(), front_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p7,
                uv: [front_tex.w(), front_tex.y()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p3,
                uv: [front_tex.x(), front_tex.y()],
                normal,
                color: WHITE,
            },
        ]);

        // triangle 3
        self.indices.extend(&[index + 4, index + 5, index + 6]);

        // triangle 4
        self.indices.extend(&[index + 4, index + 6, index + 7]);

        // Left Quad
        let normal = glm::vec3(-1., 0., 0.).into();
        self.vertices.extend(&[
            Vertex {
                pos: p0,
                uv: [left_tex.x(), left_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p1,
                uv: [left_tex.w(), left_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p3,
                uv: [left_tex.w(), left_tex.y()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p2,
                uv: [left_tex.x(), left_tex.y()],
                normal,
                color: WHITE,
            },
        ]);

        // triangle 5
        self.indices.extend(&[index + 8, index + 9, index + 10]);

        // triangle 6
        self.indices.extend(&[index + 8, index + 10, index + 11]);

        // Right Quad
        let normal = glm::vec3(1., 0., 0.).into();
        self.vertices.extend(&[
            Vertex {
                pos: p5,
                uv: [right_tex.x(), right_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p4,
                uv: [right_tex.w(), right_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p6,
                uv: [right_tex.w(), right_tex.y()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p7,
                uv: [right_tex.x(), right_tex.y()],
                normal,
                color: WHITE,
            },
        ]);

        // triangle 7
        self.indices.extend(&[index + 12, index + 13, index + 14]);

        // triangle 8
        self.indices.extend(&[index + 12, index + 14, index + 15]);

        // Bottom Quad
        let normal = glm::vec3(0., -1., 0.).into();
        self.vertices.extend(&[
            Vertex {
                pos: p0,
                uv: [bottom_tex.x(), bottom_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p4,
                uv: [bottom_tex.w(), bottom_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p5,
                uv: [bottom_tex.w(), bottom_tex.y()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p1,
                uv: [bottom_tex.x(), bottom_tex.y()],
                normal,
                color: WHITE,
            },
        ]);

        // triangle 9
        self.indices.extend(&[index + 16, index + 17, index + 18]);

        // triangle 10
        self.indices.extend(&[index + 16, index + 18, index + 19]);

        // Top Quad
        let normal = glm::vec3(0., 1., 0.).into();
        self.vertices.extend(&[
            Vertex {
                pos: p7,
                uv: [top_tex.x(), top_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p6,
                uv: [top_tex.w(), top_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p2,
                uv: [top_tex.w(), top_tex.y()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: p3,
                uv: [top_tex.x(), top_tex.y()],
                normal,
                color: WHITE,
            },
        ]);

        // triangle 11
        self.indices.extend(&[index + 20, index + 21, index + 22]);

        // triangle 12
        self.indices.extend(&[index + 20, index + 22, index + 23]);

        self
    }

    pub fn pseudocube<V>(mut self, position: V, size: [f32; 3], texture_rects: [TexRect; 6]) -> Self
    where
        V: Into<glm::Vec3>,
    {
        let pos = position.into();
        let [w, h, d] = size;
        let [back_tex, front_tex, left_tex, right_tex, bottom_tex, top_tex] = texture_rects;
        let index = self.next_index();

        // Back Quad
        let normal = glm::vec3(0., 0., -1.).into();
        self.vertices.extend(&[
            Vertex {
                pos: [pos.x + w, pos.y, pos.z],
                uv: [back_tex.x(), back_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x, pos.y, pos.z],
                uv: [back_tex.w(), back_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x, pos.y + h, pos.z],
                uv: [back_tex.w(), back_tex.y()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x + w, pos.y + h, pos.z],
                uv: [back_tex.x(), back_tex.y()],
                normal,
                color: WHITE,
            },
        ]);

        // triangle 1
        self.indices.extend(&[index, index + 1, index + 2]);

        // triangle 2
        self.indices.extend(&[index, index + 2, index + 3]);

        // Front Quad
        let normal = glm::vec3(0., 0., 1.).into();
        self.vertices.extend(&[
            Vertex {
                pos: [pos.x, pos.y, pos.z + d],
                uv: [front_tex.x(), front_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x + w, pos.y, pos.z + d],
                uv: [front_tex.w(), front_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x + w, pos.y + h, pos.z + d],
                uv: [front_tex.w(), front_tex.y()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x, pos.y + h, pos.z + d],
                uv: [front_tex.x(), front_tex.y()],
                normal,
                color: WHITE,
            },
        ]);

        // triangle 3
        self.indices.extend(&[index + 4, index + 5, index + 6]);

        // triangle 4
        self.indices.extend(&[index + 4, index + 6, index + 7]);

        // Left Quad
        let normal = glm::vec3(-1., 0., 0.).into();
        self.vertices.extend(&[
            Vertex {
                pos: [pos.x, pos.y, pos.z],
                uv: [left_tex.x(), left_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x, pos.y, pos.z + d],
                uv: [left_tex.w(), left_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x, pos.y + h, pos.z + d],
                uv: [left_tex.w(), left_tex.y()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x, pos.y + h, pos.z],
                uv: [left_tex.x(), left_tex.y()],
                normal,
                color: WHITE,
            },
        ]);

        // triangle 5
        self.indices.extend(&[index + 8, index + 9, index + 10]);

        // triangle 6
        self.indices.extend(&[index + 8, index + 10, index + 11]);

        // Right Quad
        let normal = glm::vec3(1., 0., 0.).into();
        self.vertices.extend(&[
            Vertex {
                pos: [pos.x + w, pos.y, pos.z + d],
                uv: [right_tex.x(), right_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x + w, pos.y, pos.z],
                uv: [right_tex.w(), right_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x + w, pos.y + h, pos.z],
                uv: [right_tex.w(), right_tex.y()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x + w, pos.y + h, pos.z + d],
                uv: [right_tex.x(), right_tex.y()],
                normal,
                color: WHITE,
            },
        ]);

        // triangle 7
        self.indices.extend(&[index + 12, index + 13, index + 14]);

        // triangle 8
        self.indices.extend(&[index + 12, index + 14, index + 15]);

        // Bottom Quad
        let normal = glm::vec3(0., -1., 0.).into();
        self.vertices.extend(&[
            Vertex {
                pos: [pos.x, pos.y, pos.z],
                uv: [bottom_tex.x(), bottom_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x + w, pos.y, pos.z],
                uv: [bottom_tex.w(), bottom_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x + w, pos.y, pos.z + d],
                uv: [bottom_tex.w(), bottom_tex.y()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x, pos.y, pos.z + d],
                uv: [bottom_tex.x(), bottom_tex.y()],
                normal,
                color: WHITE,
            },
        ]);

        // triangle 9
        self.indices.extend(&[index + 16, index + 17, index + 18]);

        // triangle 10
        self.indices.extend(&[index + 16, index + 18, index + 19]);

        // Top Quad
        let normal = glm::vec3(0., 1., 0.).into();
        self.vertices.extend(&[
            Vertex {
                pos: [pos.x + w, pos.y + h, pos.z],
                uv: [top_tex.x(), top_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x, pos.y + h, pos.z],
                uv: [top_tex.w(), top_tex.h()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x, pos.y + h, pos.z + d],
                uv: [top_tex.w(), top_tex.y()],
                normal,
                color: WHITE,
            },
            Vertex {
                pos: [pos.x + w, pos.y + h, pos.z + d],
                uv: [top_tex.x(), top_tex.y()],
                normal,
                color: WHITE,
            },
        ]);

        // triangle 11
        self.indices.extend(&[index + 20, index + 21, index + 22]);

        // triangle 12
        self.indices.extend(&[index + 20, index + 22, index + 23]);

        self
    }

    pub fn quad<V>(self, position: V, size: [f32; 2], colors: [Color; 4]) -> Self
    where
        V: Into<glm::Vec3>,
    {
        self.quad_with_uvs(
            position,
            size,
            colors,
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        )
    }

    pub fn quad_with_uvs<V>(
        mut self,
        position: V,
        size: [f32; 2],
        colors: [Color; 4],
        uvs: [[f32; 2]; 4],
    ) -> Self
    where
        V: Into<glm::Vec3>,
    {
        // TODO: Add argument to allow custom anchor point.

        let pos = position.into();
        let (w, h) = (size[0] / 2.0, size[1] / 2.0);
        let index = self.next_index();
        let normal = glm::vec3(0., 0., 1.).into();

        self.vertices.extend(&[
            // Bottom Left
            Vertex {
                pos: [pos.x - w, pos.y - h, pos.z],
                uv: uvs[0],
                normal,
                color: colors[0],
            },
            // Bottom Right
            Vertex {
                pos: [pos.x + w, pos.y - h, pos.z],
                uv: uvs[1],
                normal,
                color: colors[1],
            },
            // Top Right
            Vertex {
                pos: [pos.x + w, pos.y + h, pos.z],
                uv: uvs[2],
                normal,
                color: colors[2],
            },
            // Top Left
            Vertex {
                pos: [pos.x - w, pos.y + h, pos.z],
                uv: uvs[3],
                normal,
                color: colors[3],
            },
        ]);

        // triangle 1
        self.indices.extend(&[index, index + 1, index + 2]);

        // triangle 2
        self.indices.extend(&[index, index + 2, index + 3]);

        self
    }

    /// Create quad from the given points, colors and UVs.
    ///
    /// Points are provided in the order:
    ///
    ///   * Bottom Left (`-x`, `-y`)
    ///   * Bottom Right (`+x`, `-y`)
    ///   * Top Right (`+x`, `+y`)
    ///   * Top Left (`-x`, `+y`)
    ///
    /// ```ignore
    /// 3-----------2
    /// |         / |
    /// |      /    |
    /// |    /      |
    /// | /         |
    /// 0-----------1
    /// ```
    pub fn quad_with_points<V>(
        mut self,
        points: [V; 4],
        colors: [Color; 4],
        uvs: [[f32; 2]; 4],
    ) -> Self
    where
        V: Into<glm::Vec3>,
    {
        let [p1, p2, p3, p4] = points;
        let [p1, p2, p3, p4] = [p1.into(), p2.into(), p3.into(), p4.into()];
        let index = self.next_index();

        // TODO: Calculate normal
        let normal = glm::vec3(0., 0., 1.).into();

        self.vertices.extend(&[
            // Bottom Left
            Vertex {
                pos: p1.into(),
                uv: uvs[0],
                normal,
                color: colors[0],
            },
            // Bottom Right
            Vertex {
                pos: p2.into(),
                uv: uvs[1],
                normal,
                color: colors[1],
            },
            // Top Right
            Vertex {
                pos: p3.into(),
                uv: uvs[2],
                normal,
                color: colors[2],
            },
            // Top Left
            Vertex {
                pos: p4.into(),
                uv: uvs[3],
                normal,
                color: colors[3],
            },
        ]);

        // triangle 1
        self.indices.extend(&[index, index + 1, index + 2]);

        // triangle 2
        self.indices.extend(&[index, index + 2, index + 3]);

        self
    }

    /// Allocate mesh on graphics memory
    pub fn build(self, ctx: &mut GraphicContext) -> Mesh {
        let (vbuf, slice) = ctx
            .factory
            .create_vertex_buffer_with_slice(&self.vertices[..], &self.indices[..]);
        let transbuf = ctx.factory.create_constant_buffer(1);

        Mesh {
            vbuf,
            slice,
            transbuf,
        }
    }
}

#[derive(Default)]
pub struct MeshCommandBuffer(VecDeque<MeshCmd>);

impl MeshCommandBuffer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn submit(&mut self, cmd: MeshCmd) {
        self.0.push_back(cmd);
    }

    pub fn pop(&mut self) -> Option<MeshCmd> {
        self.0.pop_front()
    }
}

pub enum MeshCmd {
    AllocateMesh(Entity, MeshBuilder),
}

pub struct MeshUpkeepSystem;

impl Default for MeshUpkeepSystem {
    fn default() -> Self {
        MeshUpkeepSystem
    }
}

impl MeshUpkeepSystem {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn maintain(&self, graphics_context: &mut GraphicContext, data: MeshUpkeepData) {
        let MeshUpkeepData {
            mut mesh_cmds,
            mut meshes,
        } = data;

        while let Some(cmd) = mesh_cmds.pop() {
            use MeshCmd::*;

            match cmd {
                AllocateMesh(entity, builder) => {
                    meshes
                        .insert(entity, builder.build(graphics_context))
                        .expect("Failed to insert mesh");
                }
            }
        }
    }
}

#[derive(SystemData)]
pub struct MeshUpkeepData<'a> {
    mesh_cmds: Write<'a, MeshCommandBuffer>,
    meshes: WriteStorage<'a, Mesh>,
}
