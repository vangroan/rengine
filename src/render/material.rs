use specs::prelude::*;

use crate::{comp::GlTexture, gfx_types::GlossMaterial};

#[derive(Component)]
#[storage(DenseVecStorage)]
pub enum Material {
    Basic { texture: GlTexture },
    Lambert, // Rename to Matt
    Gloss {
        texture: GlTexture,
        material: GlossMaterial,
    },
    Gizmo,
}

#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct Gizmo;
