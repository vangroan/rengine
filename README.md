<p align="center">
⚠ Unmaintained ⚠
</p>

This engine has experienced some bitrot and no longer builds. Importantly `gfx-rs` is long dead.

# rengine

[![Build Status](https://travis-ci.org/vangroan/rengine.svg?branch=master)](https://travis-ci.org/vangroan/rengine)

Game engine/framework that aggregates several existing libraries.

## Libraries

* [glutin](https://github.com/tomaka/glutin) - Window management and OpenGL context
* [nalgebra](https://github.com/rustsim/nalgebra) - Linear algebra
* [specs](https://github.com/slide-rs/specs) - Component management, system execution

## Usage

Creating a window

```rust
extern crate rengine;

fn main() {
    let app = rengine::AppBuilder::new()
        .size(640, 480)
        .title("Example App")
        .build()
        .unwrap();

    app.run();
}
```
