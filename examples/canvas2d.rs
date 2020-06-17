#[macro_use]
extern crate specs_derive;

use log::trace;
use rengine;
use rengine::camera::CameraView;
use rengine::comp::{MeshBuilder, Transform};
use rengine::draw2d::Canvas;
use rengine::gui::{
    self, widgets, GuiGraph, GuiLayoutSystem, GuiMouseMoveSystem, GuiSortSystem, WidgetBuilder,
    WidgetEvent, WidgetEvents,
};
use rengine::res::DeltaTime;
use rengine::specs::prelude::*;
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
    widget_event_reader: shrev::ReaderId<WidgetEvent>,
}

impl Game {
    fn new(ctx: &mut Context<'_>) -> Game {
        let reader_id = ctx
            .world
            .exec(|mut widget_events: Write<'_, WidgetEvents>| widget_events.register_reader());

        Game {
            canvas: Canvas::new(&mut ctx.graphics, 640, 480).unwrap(),
            entities: vec![],
            widget_event_reader: reader_id,
        }
    }
}

impl Scene for Game {
    fn on_start(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        println!("Game on start");

        let (group_id, group_node_id) = widgets::Container::vbox()
            .with_margin([16.0, 16.0])
            .build(&mut ctx.world, &mut ctx.graphics);
        self.entities.push(group_id);
        let rows = 3;
        let columns = 5;

        for r in 0..rows {
            // Row Button Group
            let (row_btn_group_id, row_btn_grp_node_id) = widgets::Container::hbox()
                .child_of(group_node_id)
                .with_margin([8.0, 8.0])
                .build(&mut ctx.world, &mut ctx.graphics);
            self.entities.push(row_btn_group_id);

            // Buttons
            for c in 0..columns {
                let id = c + 1 + r * columns;
                let (btn_entity, _btn_id) = widgets::Button::text(format!("btn {}", id))
                    .child_of(row_btn_grp_node_id)
                    .size(64., 64.)
                    .background_image("examples/ui.png")
                    .background_src_rect([0, 0], [32, 32])
                    .tag(format!("Button {}", id))
                    .build(&mut ctx.world, &mut ctx.graphics);
                ctx.world
                    .write_storage::<Counter>()
                    .insert(btn_entity, Counter(0))
                    .unwrap();
                self.entities.push(btn_entity);
            }
        }

        let _entity = ctx
            .world
            .create_entity()
            .with(gui::GlobalPosition::new(100.0, 100.0))
            .with(gui::BoundsRect::new(100.0, 100.0))
            .with(
                gui::text::TextBatch::default()
                    .with_z(-1.)
                    .with("###############", rengine::colors::RED),
            )
            .with(Transform::default())
            .build();

        let _entity = ctx
            .world
            .create_entity()
            .with(gui::GlobalPosition::new(100.0, 100.0))
            .with(gui::BoundsRect::new(100.0, 100.0))
            .with(
                gui::text::TextBatch::default()
                    .with_z(0.0)
                    .with("XXXXXXXXXXXXXXX", rengine::colors::GREEN),
            )
            .with(Transform::default())
            .build();

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
        GuiSortSystem.run_now(&ctx.world.res);
        GuiLayoutSystem.run_now(&ctx.world.res);

        ctx.world.exec(|widget_events: Read<'_, WidgetEvents>| {
            for ev in widget_events.read(&mut self.widget_event_reader) {
                println!("Game::on_update {:?}", ev);
            }
        });

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
