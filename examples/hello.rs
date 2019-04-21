extern crate rengine;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = rengine::AppBuilder::new()
        .title("Hello Example")
        .size(500, 500)
        .background_color([0.3, 0.4, 0.5, 1.0])
        .build()?;

    app.init_scene("play");
    app.register_scene("play", |builder| {
        println!("Build scene");

        builder
    });

    app.run()?;

    Ok(())
}
