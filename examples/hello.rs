extern crate rengine;
use rengine::specs::World;
use rengine::{Scene, Trans};
use std::error::Error;

#[derive(Debug)]
struct Intro;

impl Scene for Intro {
    fn on_start(&mut self, _world: &World) -> Option<Trans> {
        println!("{:?}: On start", self);

        Trans::replace(Game)
    }

    fn on_stop(&mut self) {
        println!("{:?}: On stop", self);
    }
}

#[derive(Debug)]
struct Game;

impl Scene for Game {
    fn on_start(&mut self, _world: &World) -> Option<Trans> {
        println!("{:?}: On start", self);

        None
    }

    fn on_update(&mut self) {
        println!("{:?}: On update", self);
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
