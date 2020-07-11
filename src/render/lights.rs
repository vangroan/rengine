use gfx::traits::FactoryExt;
use nalgebra::Vector3;
use specs::prelude::*;

use crate::{
    colors::Color,
    comp::Transform,
    comp::{GlTexture, MeshBuilder},
    gfx_types,
    graphics::GraphicContext,
    render::Material,
    res::TextureAssets,
};

/// Default maximum number of lights.
pub const MAX_NUM_LIGHTS: usize = 4;

pub fn create_light<V>(
    world: &mut World,
    mut graphics: &mut GraphicContext,
    pos: V,
    debug: bool,
) -> Entity
where
    V: Into<Vector3<f32>>,
{
    // TODO: Move buffer to global resource
    let lights_buf = graphics.factory.create_constant_buffer(1);
    let texture = GlTexture::from_bundle(
        world
            .write_resource::<TextureAssets>()
            .default_texture(graphics.factory_mut()),
    );

    let mut builder = world
        .create_entity()
        .with(Transform::default().with_position(pos))
        .with(PointLight {
            buf: lights_buf,
            ambient: [0.6, 0.6, 1.0, 1.0],
            diffuse: [0.6, 0.8, 0.8, 1.0],
            specular: [1.0, 1.0, 1.0, 1.0],
        });

    builder = if debug {
        let tex_rect = texture.source_rect();
        builder
            .with(
                MeshBuilder::new()
                    .pseudocube(
                        [0.0, 0.0, 0.0],
                        [0.25, 0.25, 0.25],
                        [
                            tex_rect.clone(),
                            tex_rect.clone(),
                            tex_rect.clone(),
                            tex_rect.clone(),
                            tex_rect.clone(),
                            tex_rect,
                        ],
                    )
                    .build(&mut graphics),
            )
            .with(Material::Basic { texture })
    } else {
        builder
    };

    builder.build()
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct PointLight {
    pub buf: gfx::handle::Buffer<gfx_device::Resources, gfx_types::LightParams>,
    pub ambient: Color,
    pub diffuse: Color,
    pub specular: Color,
}

pub struct Lights {
    /// Handle to light buffer in graphics memory.
    buf: gfx::handle::Buffer<gfx_device::Resources, gfx_types::LightParams>,

    /// Maximum number of allowed lights
    max_num: usize,
}

impl Lights {
    pub fn new(graphics: &mut GraphicContext, max_num: usize) -> Self {
        Lights {
            buf: graphics.factory.create_constant_buffer(max_num),
            max_num,
        }
    }

    /// Handle to buffer of light parameters in graphics memory.
    #[inline]
    pub fn buffer(&self) -> gfx::handle::Buffer<gfx_device::Resources, gfx_types::LightParams> {
        self.buf.clone()
    }

    /// Maximum number of allowed lights.
    #[inline]
    pub fn max_num(&self) -> usize {
        self.max_num
    }
}
