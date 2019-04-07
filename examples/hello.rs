extern crate rengine;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let _app = rengine::AppBuilder::new()
        .title("Hello Example")
        .size(800, 600)
        .build();

    Ok(())
}
