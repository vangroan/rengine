use super::super::text::{TextAlignHorizontal, TextAlignVertical, TextBatch};
use super::super::{
    next_widget_tag, BoundsRect, Clickable, GlobalPosition, GuiGraph, GuiMeshBuilder, Pack,
    PackMode, Placement, WidgetBuilder, ZDepth,
};
use crate::collections::ordered_dag::NodeId;
use crate::colors::*;
use crate::comp::{GlTexture, Tag, Transform};
use crate::graphics::GraphicContext;
use crate::render::Material;
use crate::res::TextureAssets;
use nalgebra::Vector2;
use specs::prelude::*;
use std::string::ToString;

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
                    [0.1, 0.1],
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

    pub fn text<S>(text: S) -> ButtonBuilder
    where
        S: ToString,
    {
        ButtonBuilder {
            parent: None,
            tag: None,
            button_type: ButtonType::Text(text.to_string()),
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
    tag: Option<Tag>,
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

    pub fn tag<S>(mut self, name: S) -> Self
    where
        S: ToString,
    {
        self.tag = Some(Tag::new(name));
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
            tag,
            button_type,
            size,
            background,
            background_uv,
            background_src_rect,
        } = self;

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

        // Create Sprite
        let sprite_entity = world
            .create_entity()
            .with(tag.unwrap_or_else(next_widget_tag))
            .with(Button)
            .with(Pack::new(PackMode::Frame))
            .with(Placement::new(0.0, 0.0))
            .with(GlobalPosition::new(0., 0.))
            .with(ZDepth::default())
            // logical size
            .with(Transform::default())
            .with(BoundsRect::new(size[0], size[1]))
            .with(Clickable)
            // .with(Material::Basic { texture })
            .with(texture)
            .with(
                // TODO: replace with 9-patch
                GuiMeshBuilder::new()
                    .quad([0.0, 0.0], size, [WHITE, WHITE, WHITE, WHITE], uvs)
                    .build(graphics),
            )
            .build();

        let sprite_node_id = world
            .write_resource::<GuiGraph>()
            .insert_entity(sprite_entity, parent);

        // Text
        if let ButtonType::Text(text) = button_type {
            // Text (center aligned) center is the button center.
            let center = Vector2::from(size) / 2.0;
            // let center = Vector2::from(size) / 1.5;

            let text_entity = world
                .create_entity()
                .with(next_widget_tag())
                .with(Placement::from_vector(center))
                .with(GlobalPosition::default())
                .with(Transform::default())
                .with(BoundsRect::new(size[0], size[1]))
                .with(
                    TextBatch::default()
                        .with(&text, WHITE)
                        .with_z(0.0)
                        .with_align(TextAlignVertical::Center, TextAlignHorizontal::Center),
                )
                .build();

            let _text_node_id = world
                .write_resource::<GuiGraph>()
                .insert_entity(text_entity, Some(sprite_node_id));
        }

        (sprite_entity, sprite_node_id)
    }
}

#[allow(dead_code)]
enum ButtonType {
    Text(String),
    Image(GlTexture),
}
