extern crate rengine;
use rengine::Scene;
use std::error::Error;

#[derive(Debug)]
struct Intro;

impl Scene for Intro {
    fn on_start(&mut self) {
        println!("{:?}: On start", self);
    }
}

#[derive(Debug)]
struct Game;

impl Scene for Game {
    fn on_start(&mut self) {
        println!("{:?}: On start", self);
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
