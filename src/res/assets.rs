use crate::gfx_types::ColorFormat;
use gfx::traits::FactoryExt;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

/// Shared store for caching Textures
pub struct TextureAssets<'a, F: gfx::Factory<R>, R: gfx::Resources> {
    cache: BTreeMap<String, Arc<AssetBundle<R>>>,
    _marker_f: PhantomData<&'a F>,
    _marker_r: PhantomData<&'a R>,
}

impl<'a, F, R> TextureAssets<'a, F, R>
where
    F: gfx::Factory<R>,
    R: gfx::Resources,
{
    pub fn new() -> Self {
        TextureAssets {
            cache: BTreeMap::new(),
            _marker_f: PhantomData,
            _marker_r: PhantomData,
        }
    }

    /// TODO: Normalise path to something common, like absolute, or relative to CWD; for cache so we don't load same texture twice under differnet looking paths
    pub fn load_texture(&mut self, factory: &mut F, path: &str) -> Arc<AssetBundle<R>> {
        // Load from disk
        let img = image::open(path).unwrap().to_rgba();
        let (width, height) = img.dimensions();

        let kind =
            gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);

        // Mipmap data is allocated now, generated later
        let mipmap = gfx::texture::Mipmap::Allocated;

        // Allocate texture on graphics card
        let (tex, view) = factory
            .create_texture_immutable_u8::<ColorFormat>(kind, mipmap, &[&img])
            .unwrap();

        // Texture Sampler
        let sampler = factory.create_sampler_linear();

        // Cache
        let bundle = Arc::new(AssetBundle { tex, view, sampler });
        self.cache.insert(path.to_owned(), bundle.clone());

        bundle
    }
}

impl<'a, F, R> Default for TextureAssets<'a, F, R>
where
    F: gfx::Factory<R>,
    R: 'static + gfx::Resources,
{
    fn default() -> Self {
        TextureAssets::new()
    }
}

pub struct AssetBundle<R: gfx::Resources> {
    pub(crate) tex: gfx::handle::Texture<R, gfx::format::R8_G8_B8_A8>,
    pub(crate) view: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    pub(crate) sampler: gfx::handle::Sampler<R>,
}
