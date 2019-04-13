extern crate rengine;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let app = rengine::AppBuilder::new()
        .title("Hello Example")
        .size(800, 600)
        .background_color([0.3, 0.4, 0.5, 1.0])
        .build()?;

    app.run()?;

    Ok(())
}
