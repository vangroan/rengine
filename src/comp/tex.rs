use crate::res::AssetBundle;
use gfx_device::Resources;
use specs::{Component, DenseVecStorage};
use std::sync::Arc;

// TODO: Consider renaming to TextureSampler, TextureHandle or ImmutableTexture

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct GlTexture {
    pub(crate) bundle: Arc<AssetBundle<Resources>>,
}

impl GlTexture {
    pub fn from_bundle(bundle: Arc<AssetBundle<Resources>>) -> Self {
        GlTexture { bundle }
    }
}
