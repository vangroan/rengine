use crate::colors::Color;
use crate::gfx_types::{Transform, Vertex};
use crate::graphics::GraphicContext;
use gfx::handle::Buffer;
use gfx::traits::FactoryExt;
use gfx::Slice;
use specs::{Component, DenseVecStorage};

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

    pub fn quad<V>(mut self, position: V, size: [f32; 2], colors: [Color; 4]) -> Self
    where
        V: Into<glm::Vec3>,
    {
        let pos = position.into();
        let (w, h) = (size[0], size[1]);

        // New vertices will be inserted starting here
        let index = if self.vertices.len() > 0 {
            self.vertices.len() - 1
        } else {
            0
        } as u16;

        self.vertices.extend(&[
            Vertex {
                pos: [pos.x, pos.y, pos.z],
                color: colors[0],
            },
            Vertex {
                pos: [pos.x + w, pos.y, pos.z],
                color: colors[1],
            },
            Vertex {
                pos: [pos.x + w, pos.y + h, pos.z],
                color: colors[2],
            },
            Vertex {
                pos: [pos.x, pos.y + h, pos.z],
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