use gfx::traits::FactoryExt;
use nalgebra::Vector3;
use specs::prelude::*;

use crate::{colors::Color, comp::Transform, gfx_types, graphics::GraphicContext};

/// Default maximum number of lights.
pub const MAX_NUM_LIGHTS: usize = 4;

pub fn create_light<V>(world: &mut World, graphics: &mut GraphicContext, pos: V) -> Entity
where
    V: Into<Vector3<f32>>,
{
    // TODO: Move buffer to global resource
    let lights_buf = graphics.factory.create_constant_buffer(1);

    world
        .create_entity()
        .with(Transform::default().with_position(pos))
        .with(PointLight {
            buf: lights_buf,
            ambient: [0.6, 0.6, 1.0, 1.0],
            diffuse: [0.6, 0.8, 0.8, 1.0],
            specular: [1.0, 1.0, 1.0, 1.0],
        })
        .build()
}

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct PointLight {
    pub(crate) buf: gfx::handle::Buffer<gfx_device::Resources, gfx_types::LightParams>,
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
