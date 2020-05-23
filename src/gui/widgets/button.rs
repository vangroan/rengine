use super::super::text::TextBatch;
use super::super::{
    BoundsRect, GlobalPosition, GuiGraph, GuiMeshBuilder, GuiSettings, Pack, PackMode, Placement,
    WidgetBuilder,
};
use crate::collections::ordered_dag::NodeId;
use crate::colors::*;
use crate::comp::{GlTexture, Transform};
use crate::graphics::GraphicContext;
use crate::render::Material;
use crate::res::TextureAssets;
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
            background_src_rect: None,
        }
    }
}

#[must_use = "Call .build() on widget builder."]
pub struct ButtonBuilder {
    parent: Option<NodeId>,
    button_type: ButtonType,
    size: [f32; 2],
    background: Option<String>,
    background_uv: [[f32; 2]; 4],
    background_src_rect: Option<[Vector2<u32>; 2]>,
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

    pub fn background_src_rect<V>(mut self, pos: V, size: V) -> Self
    where
        V: Into<Vector2<u32>>,
    {
        self.background_src_rect = Some([pos.into(), size.into()]);
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
            background_src_rect,
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

        let uvs = if let Some([pos, size]) = background_src_rect {
            texture.source_rect().sub_rect(pos, size).into()
        } else {
            background_uv
        };

        let graphics_size = Vector2::from(size) / pixel_scale;

        // Create Sprite
        let sprite_entity = world
            .create_entity()
            .with(Button)
            .with(Pack::new(PackMode::Frame))
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
                        uvs,
                    )
                    .build(graphics),
            )
            .build();

        let sprite_node_id = world
            .write_resource::<GuiGraph>()
            .insert_entity(sprite_entity, parent);

        // Text
        if let ButtonType::Text(text) = button_type {
            let text_entity = world
                .create_entity()
                .with(Placement::new(0.0, 0.0))
                .with(GlobalPosition::default())
                .with(Transform::default())
                .with(BoundsRect::new(size[0], size[1]))
                .with(TextBatch::default().with(&text, WHITE))
                .build();

            let _text_node_id = world
                .write_resource::<GuiGraph>()
                .insert_entity(text_entity, Some(sprite_node_id));
        }

        (sprite_entity, sprite_node_id)
    }
}

enum ButtonType {
    Text(String),
    Image(GlTexture),
}
