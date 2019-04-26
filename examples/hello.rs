extern crate rengine;
use rengine::angle::Deg;
use rengine::comp::{GlTexture, MeshBuilder, Transform, X_AXIS, Y_AXIS};
use rengine::specs::{Builder, Entity};
use rengine::{Context, GlTextureAssets, Scene, Trans};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct Intro;

impl Scene for Intro {
    fn on_start(&mut self, _ctx: &mut Context<'_>) -> Option<Trans> {
        println!("{:?}: On start", self);

        Trans::replace(Game::default())
    }

    fn on_stop(&mut self, _ctx: &mut Context<'_>) -> Option<Trans> {
        println!("{:?}: On stop", self);

        None
    }
}

#[derive(Debug, Default)]
struct Game {
    entities: Vec<Entity>,
}

impl Scene for Game {
    fn on_start(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        println!("{}: On start", self);

        // Test Pseudocube
        let tex = GlTexture::from_bundle(
            ctx.world
                .write_resource::<GlTextureAssets>()
                .load_texture(&mut ctx.graphics.factory_mut(), "examples/block.png"),
        );
        let tex_rects = {
            let tex_rect = tex.source_rect();
            let back_rect = tex_rect.sub_rect([0, 0], [16, 16]);
            let front_rect = tex_rect.sub_rect([16, 0], [16, 16]);
            let left_rect = tex_rect.sub_rect([32, 0], [16, 16]);
            let right_rect = tex_rect.sub_rect([0, 16], [16, 16]);
            let bottom_rect = tex_rect.sub_rect([16, 16], [16, 16]);
            let top_rect = tex_rect.sub_rect([32, 16], [16, 16]);
            [
                back_rect,
                front_rect,
                left_rect,
                right_rect,
                bottom_rect,
                top_rect,
            ]
        };
        let entity = ctx
            .world
            .create_entity()
            .with(
                MeshBuilder::new()
                    // .quad(
                    //     [0., 0., 0.],
                    //     [1., 1.],
                    //     // [colors::RED, colors::GREEN, colors::BLUE, colors::MAGENTA],
                    //     [colors::WHITE, colors::WHITE, colors::WHITE, colors::WHITE],
                    // )
                    .pseudocube([0., 0., 0.], [1., 1., 1.], tex_rects)
                    .build(&mut ctx.graphics),
            )
            .with(
                Transform::default()
                    .with_anchor([0.5, 0.5, 0.5])
                    .with_position([0.25, 0.25, 0.])
                    .with_scale([0.5, 0.5, 0.5])
                    .with_rotate_world(Deg(45.), Y_AXIS)
                    .with_rotate_world(Deg(30.), X_AXIS),
            )
            .with(tex)
            .build();

        self.entities.push(entity);

        None
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        println!("{}: On stop", self);

        if let Err(err) = ctx.world.delete_entities(&self.entities) {
            panic!(err);
        }

        None
    }

    fn on_update(&mut self) {
        println!("{}: On update", self);
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Game")
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = rengine::AppBuilder::new()
        .title("Hello Example")
        .size(500, 500)
        .background_color([0.3, 0.4, 0.5, 1.0])
        .init_scene(Intro)
        .build()?;

    app.run()?;

    Ok(())
}
