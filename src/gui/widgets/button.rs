use crate::colors::*;
use crate::comp::{GlTexture, Transform};
use crate::graphics::GraphicContext;
use crate::gui::GuiMeshBuilder;
use crate::render::Material;
use crate::res::TextureAssets;
use specs::{Builder, Component, DenseVecStorage, EntityBuilder};

#[derive(Component)]
#[storage(DenseVecStorage)]
pub struct Button;

impl Button {
    pub fn bundle<'a>(
        builder: EntityBuilder<'a>,
        graphics_context: &mut GraphicContext,
    ) -> EntityBuilder<'a> {
        let texture = GlTexture::from_bundle(
            builder
                .world
                .write_resource::<TextureAssets>()
                .default_texture(graphics_context.factory_mut()),
        );
        builder
            .with(Button)
            .with(Transform::default().with_position([0.0, 0.0, 0.0]))
            .with(Material::Basic { texture })
            .with(
                // TODO: replace with 9-patch
                GuiMeshBuilder::new()
                    .quad(
                        [0.0, 0.0],
                        [0.01, 0.01],
                        [GREEN, GREEN, GREEN, GREEN],
                        [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                    )
                    .build(graphics_context),
            )
    }
}
