use gfx::traits::FactoryExt;
use nalgebra::Vector3;
use specs::prelude::*;

use crate::{colors::Color, comp::Transform, gfx_types, graphics::GraphicContext};

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