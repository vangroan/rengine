#[macro_use]
extern crate specs_derive;

use log::trace;
use rengine;
use rengine::camera::CameraView;
use rengine::comp::{MeshBuilder, Transform};
use rengine::draw2d::Canvas;
use rengine::gui::{self, widgets};
use rengine::gui::{GuiGraph, GuiLayoutSystem, GuiMouseMoveSystem, WidgetBuilder};
use rengine::res::DeltaTime;
use rengine::specs::{
    Builder, Component, DenseVecStorage, Entity, Join, Read, ReadStorage, RunNow, WriteStorage,
};
use rengine::{Context, Scene, Trans};
use std::error::Error;

#[derive(Component)]
#[storage(DenseVecStorage)]
struct Counter(u32);

#[derive(Debug)]
struct Intro;

impl Scene for Intro {
    fn on_start(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        println!("{:?}: On start", self);

        ctx.world.register::<Counter>();

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

        let btn_group_id = widgets::create_hbox(&mut ctx.world);
        let btn_grp_node_id = ctx
            .world
            .write_resource::<GuiGraph>()
            .insert_entity(btn_group_id, None);
        self.entities.push(btn_group_id);

        for i in 0..4 {
            let (btn_entity, _btn_id) = widgets::Button::text(&format!("Click Me {}", i))
                .child_of(btn_grp_node_id)
                .background_image("examples/ui.png")
                .background_src_rect([0, 0], [32, 32])
                .build(&mut ctx.world, &mut ctx.graphics);
            ctx.world
                .write_storage::<Counter>()
                .insert(btn_entity, Counter(0))
                .unwrap();
            self.entities.push(btn_entity);
        }

        println!("entitites {:?}", self.entities);
        ctx.world.read_resource::<GuiGraph>().debug_print();

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

        GuiMouseMoveSystem.run_now(&ctx.world.res);
        GuiLayoutSystem.run_now(&ctx.world.res);

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
