use gfx::traits::FactoryExt;
use specs::prelude::*;

use crate::{colors::Color, comp::GlTexture, gfx_types, graphics::GraphicContext};

#[derive(Component)]
#[storage(DenseVecStorage)]
pub enum Material {
    Basic {
        texture: GlTexture,
    },
    Lambert, // Rename to Matt
    Gloss {
        texture: GlTexture,
        material: GlossMaterial,
    },
    Gizmo,
}

#[derive(Debug, Clone)]
pub struct GlossMaterial {
    /// Handle to material buffer in graphics memory.
    pub(crate) material_buf: gfx::handle::Buffer<gfx_device::Resources, gfx_types::GlossMaterial>,
    pub ambient: Color,
    pub diffuse: Color,
    pub specular: Color,
    pub shininess: f32,
}

impl GlossMaterial {
    pub fn new(
        graphics: &mut GraphicContext,
        ambient: Color,
        diffuse: Color,
        specular: Color,
        shininess: f32,
    ) -> Self {
        GlossMaterial {
            material_buf: graphics.factory.create_constant_buffer(1),
            ambient,
            diffuse,
            specular,
            shininess,
        }
    }
}

impl Into<gfx_types::GlossMaterial> for GlossMaterial {
    fn into(self) -> gfx_types::GlossMaterial {
        gfx_types::GlossMaterial {
            ambient: self.ambient.into(),
            diffuse: self.diffuse.into(),
            specular: self.specular.into(),
            shininess: self.shininess,
        }
    }
}

#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct Gizmo;
