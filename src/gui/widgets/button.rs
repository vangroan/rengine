use super::super::{GuiMeshBuilder, Placement, WidgetBounds};
use crate::colors::*;
use crate::comp::{GlTexture, Transform};
use crate::graphics::GraphicContext;
use crate::render::Material;
use crate::res::TextureAssets;
use specs::{Builder, Component, DenseVecStorage, Entity, EntityBuilder, World};

pub fn create_text_button(world: &mut World, graphics: &mut GraphicContext, _text: &str) -> Entity {
    let _texture = GlTexture::from_bundle(
        world
            .write_resource::<TextureAssets>()
            .default_texture(graphics.factory_mut()),
    );

    // Create Text

    // Create Sprite
    world.create_entity().build()
}

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Button;

impl Button {
    pub fn bundle<'a>(
        builder: EntityBuilder<'a>,
        graphics_context: &mut GraphicContext,
    ) -> EntityBuilder<'a> {
        // TODO: Need pixel scale and dpi to determine mesh size.
        let texture = GlTexture::from_bundle(
            builder
                .world
                .write_resource::<TextureAssets>()
                .default_texture(graphics_context.factory_mut()),
        );
        builder
            .with(Button)
            .with(Placement::zero())
            .with(Transform::default().with_position([0.0, 0.0, 0.0]))
            .with(WidgetBounds::new(100.0, 100.0))
            .with(Material::Basic { texture })
            .with(
                // TODO: replace with 9-patch
                GuiMeshBuilder::new()
                    .quad(
                        [0.0, 0.0],
                        [0.1, 0.1], // logical size / 1000.0 for now
                        [GREEN, GREEN, GREEN, GREEN],
                        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                    )
                    .build(graphics_context),
            )
    }
}
