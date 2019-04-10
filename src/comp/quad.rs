use glm::Vec3;
use specs::prelude::*;

use crate::colors::{Color, WHITE};
use crate::gfx_types::Vertex;

/// A basic flat, rectangular plane
#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Quad {
    pos: Vec3,
    size: [f32; 2],
    color: Color,
}

impl Quad {
    pub fn new<V>(pos: V, size: [f32; 2]) -> Self
    where
        V: Into<Vec3>,
    {
        Quad {
            pos: pos.into(),
            size,
            color: WHITE,
        }
    }

    /// Creates mesh vertices and indices using the Quad's properties
    pub fn create_vertices_indices(&self) -> (Vec<Vertex>, Vec<u16>) {
        use crate::colors::{BLUE, GREEN, MAGENTA, RED};
        let (mut vs, mut is) = (vec![], vec![]);
        let &Quad {
            ref pos, ref size, ..
        } = self;
        let (w, h) = (size[0], size[1]);

        vs.extend(&[
            Vertex {
                pos: [pos.x, pos.y, pos.z],
                color: RED,
            },
            Vertex {
                pos: [pos.x + w, pos.y, pos.z],
                color: GREEN,
            },
            Vertex {
                pos: [pos.x + w, pos.y + h, pos.z],
                color: BLUE,
            },
            Vertex {
                pos: [pos.x, pos.y + h, pos.z],
                color: MAGENTA,
            },
        ]);

        // triangle 1
        is.extend(&[0, 1, 2]);

        // triangle 2
        is.extend(&[0, 2, 3]);

        (vs, is)
    }
}
