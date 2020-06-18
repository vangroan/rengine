use crate::comp::GlTexture;
use specs::{Component, DenseVecStorage, FlaggedStorage};

#[derive(Component)]
#[storage(DenseVecStorage)]
pub enum Material {
    Basic { texture: GlTexture },
    Lambert, // Rename to Matt
    Phong,   // Rename to Gloss
    Gizmo,
}

#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct Gizmo;
