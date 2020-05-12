use log::trace;
use rengine;
use rengine::camera::CameraView;
use rengine::comp::{MeshBuilder, Transform};
use rengine::draw2d::Canvas;
use rengine::gui::{self, widgets};
use rengine::res::DeltaTime;
use rengine::specs::{Builder, Entity, Join, Read, ReadStorage, WriteStorage};
use rengine::{Context, Scene, Trans};
use std::error::Error;

#[derive(Debug)]
struct Intro;

impl Scene for Intro {
    fn on_start(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        println!("{:?}: On start", self);

        Trans::replace(Game::new(ctx))
    }

    fn on_stop(&mut self, _ctx: &mut Context<'_>) -> Option<Trans> {
        println!("{:?}: On stop", self);

        None
    }
}

struct Game {
    canvas: Canvas,
    entities: Vec<Entity>,
}

impl Game {
    fn new(ctx: &mut Context<'_>) -> Game {
        Game {
            canvas: Canvas::new(&mut ctx.graphics, 640, 480).unwrap(),
            entities: vec![],
        }
    }
}

impl Scene for Game {
    fn on_start(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        println!("Game on start");

        self.entities
            .push(widgets::Button::bundle(ctx.world.create_entity(), &mut ctx.graphics).build());
        println!("entitites {:?}", self.entities);

        None
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        println!("Game on stop");

        if let Err(err) = ctx.world.delete_entities(&self.entities) {
            panic!(err);
        }

        self.entities.clear();

        None
    }

    fn on_update(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        ctx.world.exec(
            |(_cam, mut trans, dt): (
                ReadStorage<'_, CameraView>,
                WriteStorage<'_, Transform>,
                Read<'_, DeltaTime>,
            )| {
                for trans in (&mut trans).join() {
                    // trans.translate([0.0, 0.0, -10.0 * dt.as_secs_float()]);
                }
            },
        );

        None
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = rengine::AppBuilder::new()
        .title("Canvas 2D Example")
        .size(640, 480)
        .background_color([0.3, 0.4, 0.5, 1.0])
        .init_scene(Intro)
        .build()?;

    app.run()?;

    Ok(())
}
