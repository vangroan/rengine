use crate::comp::GlTexture;
use specs::{Component, DenseVecStorage};

#[derive(Component)]
#[storage(DenseVecStorage)]
pub enum Material {
    Basic { texture: GlTexture },
    Phong,
    Gizmo,
}
