extern crate rengine;

use rengine::colors::{BLACK, WHITE};
use rengine::gui::text::TextBatch;
use rengine::specs::{Builder, Entity};
use rengine::{AppBuilder, Context, Scene, Trans};

struct Game {
    entities: Vec<Entity>,
}

impl Game {
    fn new() -> Self {
        Game { entities: vec![] }
    }
}

impl Scene for Game {
    fn on_start(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        let entity = ctx
            .world
            .create_entity()
            .with(TextBatch::new().with("Hello, World", WHITE))
            .build();

        self.entities.push(entity);

        None
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        let maybe_err = ctx.world.delete_entities(&self.entities).err();

        if let Some(err) = maybe_err {
            panic!(err);
        }

        self.entities.clear();

        None
    }

    fn on_update(&mut self, _ctx: &mut Context<'_>) -> Option<Trans> {
        None
    }
}

fn main() {
    let app = AppBuilder::new()
        .title("Text Example")
        .size(800, 600)
        .background_color(BLACK)
        .init_scene(Game::new())
        .build()
        .expect("Failed to build application");

    app.run().expect("Failure during main loop");
}
