use crate::gfx_types::ColorFormat;
use gfx::texture::{FilterMethod, SamplerInfo, WrapMode};
use gfx_device::{Factory, Resources};
use std::collections::BTreeMap;

use std::sync::Arc;

const DEFAULT_TEXTURE_KEY: &str = "#";

/// Shared store for caching Textures
///
/// Inner values are protected by Arc, but the container
/// is not thread safe. This is so that textures, which
/// are immutable, can be sent across thread boundries to
/// systems, but access to the cache itself must occur from
/// a single thread.
pub struct TextureAssets {
    /// Reference counted shared textures.
    cache: BTreeMap<String, Arc<AssetBundle>>,
}

impl TextureAssets {
    pub fn new() -> Self {
        TextureAssets {
            cache: BTreeMap::new(),
        }
    }

    /// Retrieve the special default texture.
    ///
    /// The default texture is a 1x1 white pixel, so a mesh with no texture
    /// can be drawn using a shader that expects a texture to be loaded.
    ///
    /// Sampling an empty texture would be undefined behaviour.
    pub fn default_texture(&mut self, factory: &mut Factory) -> Arc<AssetBundle> {
        // Constant image
        let data: &[&[u8]] = &[&[0xFF, 0xFF, 0xFF, 0xFF]];
        let (width, height) = (1, 1);

        self.create_texture(factory, DEFAULT_TEXTURE_KEY, width, height, data)
    }

    /// TODO: Normalise path to something common, like absolute, or relative to CWD; for cache so we don't load same texture twice under differnet looking paths
    pub fn load_texture(&mut self, factory: &mut Factory, path: &str) -> Arc<AssetBundle> {
        // Load from disk
        let img = image::open(path).unwrap().to_rgba();
        let (width, height) = img.dimensions();

        self.create_texture(factory, path, width, height, &[&img])
    }

    /// Creates a texture in the cache.
    ///
    /// The key is the unique identifier of the texture.
    ///
    /// The width and height are the dimensions of the image, and the data
    /// is a slice of pixels, represented as slices.
    fn create_texture(
        &mut self,
        factory: &mut Factory,
        key: &str,
        width: u32,
        height: u32,
        data: &[&[u8]],
    ) -> Arc<AssetBundle> {
        self.cache
            .entry(key.to_owned())
            .or_insert_with(|| {
                let kind = gfx::texture::Kind::D2(
                    width as u16,
                    height as u16,
                    gfx::texture::AaMode::Single,
                );

                // Mipmap data is allocated now, generated later
                let mipmap = gfx::texture::Mipmap::Allocated;

                // Allocate texture on graphics card
                let (tex, view) = gfx::Factory::create_texture_immutable_u8::<ColorFormat>(
                    factory, kind, mipmap, data,
                )
                .unwrap();

                // Texture Sampler
                // let sampler = factory.create_sampler_linear();
                let sampler = gfx::Factory::create_sampler(
                    factory,
                    SamplerInfo::new(FilterMethod::Scale, WrapMode::Clamp),
                );

                // Cache
                Arc::new(AssetBundle {
                    tex_size: (width, height),
                    _tex: tex,
                    view,
                    sampler,
                })
            })
            .clone()
    }
}

impl Default for TextureAssets {
    fn default() -> Self {
        TextureAssets::new()
    }
}

pub struct AssetBundle {
    pub(crate) tex_size: (u32, u32),
    _tex: gfx::handle::Texture<Resources, gfx::format::R8_G8_B8_A8>,
    pub(crate) view: gfx::handle::ShaderResourceView<Resources, [f32; 4]>,
    pub(crate) sampler: gfx::handle::Sampler<Resources>,
}
