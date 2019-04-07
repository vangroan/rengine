extern crate rengine;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = rengine::AppBuilder::new()
        .title("Hello Example")
        .size(800, 600)
        .build()?;

    app.run();

    Ok(())
}
