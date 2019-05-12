use crate::res::AssetBundle;
use nalgebra::Vector2;
use specs::{Component, DenseVecStorage};
use std::sync::Arc;

// TODO: Consider renaming to TextureSampler, TextureHandle or ImmutableTexture

#[derive(Component, Clone)]
#[storage(DenseVecStorage)]
pub struct GlTexture {
    pub(crate) bundle: Arc<AssetBundle>,
}

impl GlTexture {
    pub fn from_bundle(bundle: Arc<AssetBundle>) -> Self {
        GlTexture { bundle }
    }

    pub fn source_rect(&self) -> TexRect {
        let (width, height) = self.bundle.as_ref().tex_size;

        TexRect {
            pixel_size: Vector2::new(width, height),
            pos: Vector2::new(0., 0.),
            size: Vector2::new(1., 1.),
        }
    }
}

pub struct TexRect {
    pixel_size: Vector2<u32>,
    pos: Vector2<f32>,
    size: Vector2<f32>,
}

impl TexRect {
    /// Creates a new rectangle given pixel coordinates
    pub fn sub_rect<V>(&self, pos: V, size: V) -> TexRect
    where
        V: Into<Vector2<u32>>,
    {
        // Convert to float
        let new_pixel_pos = pos.into();
        let new_pixel_size = size.into();

        let (pw, ph) = (self.pixel_size.x as f32, self.pixel_size.y as f32);
        let (npx, npy) = (new_pixel_pos.x as f32, new_pixel_pos.y as f32);
        let (npw, nph) = (new_pixel_size.x as f32, new_pixel_size.y as f32);

        // Convert pixel coordinates to texture coordinates
        let (x, y): (f32, f32) = (npx / pw, npy / ph);
        let (w, h): (f32, f32) = ((npx + npw) / pw, (npy + nph) / ph);

        TexRect {
            // Store pixel size for next sub rectangle
            pixel_size: Vector2::new(new_pixel_size.x, new_pixel_size.y),
            pos: Vector2::new(x, y),
            size: Vector2::new(w, h),
        }
    }

    #[inline]
    pub fn x(&self) -> f32 {
        self.pos.x
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.pos.y
    }

    #[inline]
    pub fn w(&self) -> f32 {
        self.size.x
    }

    #[inline]
    pub fn h(&self) -> f32 {
        self.size.y
    }
}
