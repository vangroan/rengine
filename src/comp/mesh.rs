use crate::colors::{Color, WHITE};
use crate::comp::TexRect;
use crate::gfx_types::{Transform, Vertex};
use crate::graphics::GraphicContext;
use gfx::handle::Buffer;
use gfx::traits::FactoryExt;
use gfx::Slice;
use specs::{Component, DenseVecStorage};

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

impl MeshBuilder {
    pub fn new() -> Self {
        MeshBuilder {
            vertices: vec![],
            indices: vec![],
        }
    }

    // New vertices will be inserted starting here
    fn next_index(&self) -> u16 {
        if self.vertices.len() > 0 {
            (self.vertices.len() - 1) as u16
        } else {
            0
        }
    }

    pub fn pseudocube<V>(mut self, position: V, size: [f32; 3], texture_rects: [TexRect; 6]) -> Self
    where
        V: Into<glm::Vec3>,
    {
        let pos = position.into();
        let [w, h, d] = size;
        let [back_tex, front_tex, left_tex, right_tex, _top_tex, _bottom_tex] = texture_rects;
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

        self
    }

    pub fn quad<V>(mut self, position: V, size: [f32; 2], colors: [Color; 4]) -> Self
    where
        V: Into<glm::Vec3>,
    {
        let pos = position.into();
        let (w, h) = (size[0], size[1]);
        let index = self.next_index();
        let normal = glm::vec3(0., 0., 1.).into();

        self.vertices.extend(&[
            Vertex {
                pos: [pos.x, pos.y, pos.z],
                uv: [0.0, 0.0],
                normal,
                color: colors[0],
            },
            Vertex {
                pos: [pos.x + w, pos.y, pos.z],
                uv: [1.0, 0.0],
                normal,
                color: colors[1],
            },
            Vertex {
                pos: [pos.x + w, pos.y + h, pos.z],
                uv: [1.0, 1.0],
                normal,
                color: colors[2],
            },
            Vertex {
                pos: [pos.x, pos.y + h, pos.z],
                uv: [0.0, 1.0],
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
