use super::super::{
    BoundsRect, GlobalPosition, GuiGraph, GuiMeshBuilder, GuiSettings, Placement, WidgetBuilder,
};
use crate::collections::ordered_dag::NodeId;
use crate::colors::*;
use crate::comp::{GlTexture, Transform};
use crate::graphics::GraphicContext;
use crate::render::Material;
use crate::res::{DeviceDimensions, TextureAssets};
use nalgebra::Vector2;
use specs::{Builder, Component, DenseVecStorage, Entity, EntityBuilder, World};

pub fn create_text_button(
    world: &mut World,
    graphics: &mut GraphicContext,
    _text: &str,
    parent: Option<NodeId>,
) -> Entity {
    let texture = GlTexture::from_bundle(
        world
            .write_resource::<TextureAssets>()
            .default_texture(graphics.factory_mut()),
    );

    // Create Text

    // Create Sprite
    let sprite_entity = world
        .create_entity()
        .with(Button)
        .with(Placement::new(0.0, 0.0))
        .with(GlobalPosition::new(0., 0.))
        .with(Transform::default())
        .with(BoundsRect::new(100.0, 100.0))
        .with(Material::Basic { texture })
        .with(
            // TODO: replace with 9-patch
            GuiMeshBuilder::new()
                .quad(
                    [0.0, 0.0],
                    [0.1, 0.1], // logical size / 1000.0 for now
                    [WHITE, WHITE, WHITE, WHITE],
                    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                )
                .build(graphics),
        )
        .build();

    let _sprite_node = world
        .write_resource::<GuiGraph>()
        .insert_entity(sprite_entity, parent);

    sprite_entity
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
            .with(Placement::new(0.5, 0.5))
            .with(Transform::default().with_position([0.0, 0.0, 0.0]))
            .with(BoundsRect::new(100.0, 100.0))
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

    pub fn text(text: &str) -> ButtonBuilder {
        ButtonBuilder {
            parent: None,
            button_type: ButtonType::Text(text.to_owned()),
            size: [100.0, 100.0],
            background: None,
            background_uv: [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        }
    }
}

pub struct ButtonBuilder {
    parent: Option<NodeId>,
    button_type: ButtonType,
    size: [f32; 2],
    background: Option<String>,
    background_uv: [[f32; 2]; 4],
}

impl ButtonBuilder {
    pub fn child_of(mut self, parent: NodeId) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn size(mut self, x: f32, y: f32) -> Self {
        self.size = [x, y];
        self
    }

    pub fn background_image(mut self, file_path: &str) -> Self {
        self.background = Some(file_path.to_owned());
        self
    }

    pub fn background_uv(mut self, uvs: [[f32; 2]; 4]) -> Self {
        self.background_uv = uvs;
        self
    }
}

impl WidgetBuilder for ButtonBuilder {
    fn build(self, world: &mut World, graphics: &mut GraphicContext) -> (Entity, NodeId) {
        let ButtonBuilder {
            parent,
            button_type,
            size,
            background,
            background_uv,
        } = self;

        let pixel_scale = world.read_resource::<GuiSettings>().pixel_scale;

        let texture = match background {
            Some(file_path) => GlTexture::from_bundle(
                world
                    .write_resource::<TextureAssets>()
                    .load_texture(graphics.factory_mut(), &file_path),
            ),
            None => GlTexture::from_bundle(
                world
                    .write_resource::<TextureAssets>()
                    .default_texture(graphics.factory_mut()),
            ),
        };

        let graphics_size = Vector2::from(size) / pixel_scale;

        // Create Sprite
        let sprite_entity = world
            .create_entity()
            .with(Button)
            .with(Placement::new(0.0, 0.0))
            .with(GlobalPosition::new(0., 0.))
            // logical size
            .with(Transform::default())
            .with(BoundsRect::new(size[0], size[1]))
            .with(Material::Basic { texture })
            .with(
                // TODO: replace with 9-patch
                GuiMeshBuilder::new()
                    .quad(
                        [0.0, 0.0],
                        graphics_size.into(),
                        [WHITE, WHITE, WHITE, WHITE],
                        background_uv,
                    )
                    .build(graphics),
            )
            .build();

        let sprite_node_id = world
            .write_resource::<GuiGraph>()
            .insert_entity(sprite_entity, parent);
        (sprite_entity, sprite_node_id)
    }
}

enum ButtonType {
    Text(String),
    Image(GlTexture),
}
