use crate::colors::Color;
use crate::gfx_types::{Transform, Vertex};
use crate::graphics::GraphicContext;
use gfx::handle::Buffer;
use gfx::traits::FactoryExt;
use gfx::Slice;
use specs::{Component, DenseVecStorage};

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct GuiMesh {
    pub(crate) vbuf: Buffer<gfx_device::Resources, Vertex>,
    pub(crate) slice: Slice<gfx_device::Resources>,
    pub(crate) transbuf: Buffer<gfx_device::Resources, Transform>,
}

pub struct GuiMeshBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Default for GuiMeshBuilder {
    fn default() -> Self {
        GuiMeshBuilder {
            vertices: vec![],
            indices: vec![],
        }
    }
}

impl GuiMeshBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    /// New vertices will be inserted starting here
    #[inline]
    fn next_index(&self) -> u16 {
        self.vertices.len() as u16
    }

    /// Creates a quad with an origin at the bottom left.
    pub fn quad<V>(
        mut self,
        position: V,
        size: [f32; 2],
        colors: [Color; 4],
        uvs: [[f32; 2]; 4],
    ) -> Self
    where
        V: Into<glm::Vec2>,
    {
        let pos = position.into();
        // let (w, h) = (size[0] / 2.0, size[1] / 2.0);
        let [w, h] = size;
        let index = self.next_index();

        // GUI Mesh is ordered differently from regular mesh. UVs are applied in
        // reverse order.
        self.vertices
            .push(vertex([pos.x, pos.y], uvs[3], colors[0]));
        self.vertices
            .push(vertex([pos.x + w, pos.y], uvs[2], colors[1]));
        self.vertices
            .push(vertex([pos.x + w, pos.y + h], uvs[1], colors[2]));
        self.vertices
            .push(vertex([pos.x, pos.y + h], uvs[0], colors[3]));

        // GUI elements have their y-axis inverted in the shader. Because of
        // backface culling, the winding is clockwise instead of the usual
        // anti-clockwise.
        self.indices.extend(&[index, index + 2, index + 1]);
        self.indices.extend(&[index, index + 3, index + 2]);

        self
    }

    pub fn nine_patch(self) -> Self {
        unimplemented!()
    }

    pub fn build(self, ctx: &mut GraphicContext) -> GuiMesh {
        let (vbuf, slice) = ctx
            .factory
            .create_vertex_buffer_with_slice(&self.vertices[..], &self.indices[..]);
        let transbuf = ctx.factory.create_constant_buffer(1);

        GuiMesh {
            vbuf,
            slice,
            transbuf,
        }
    }
}

#[inline]
fn vertex<V>(position: V, uv: [f32; 2], color: Color) -> Vertex
where
    V: Into<glm::Vec2>,
{
    let p = position.into();
    Vertex {
        pos: [p.x, p.y, 0.0],
        uv,
        normal: [0.0, 0.0, 1.0],
        color,
    }
}
