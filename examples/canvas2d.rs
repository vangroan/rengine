use log::trace;
use rengine;
use rengine::draw2d::Canvas;
use rengine::{Context, Scene, Trans};
use std::error::Error;

#[derive(Debug)]
struct Intro;

impl Scene for Intro {
    fn on_start(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        trace!("{:?}: On start", self);

        Trans::replace(Game::new(ctx))
    }

    fn on_stop(&mut self, _ctx: &mut Context<'_>) -> Option<Trans> {
        trace!("{:?}: On stop", self);

        None
    }
}

struct Game {
    canvas: Canvas,
}

impl Game {
    fn new(ctx: &mut Context<'_>) -> Game {
        Game {
            canvas: Canvas::new(&mut ctx.graphics, 640, 480).unwrap(),
        }
    }
}

impl Scene for Game {}

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
