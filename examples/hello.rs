extern crate rengine;

use rengine::angle::Deg;
use rengine::comp::{X_AXIS, Y_AXIS};
use rengine::specs;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = rengine::AppBuilder::new()
        .title("Hello Example")
        .size(500, 500)
        .background_color([0.3, 0.4, 0.5, 1.0])
        .build()?;

    app.init_scene("example");
    app.register_scene("example", |builder| {
        println!("Building example scene");

        // Test Quad
        // use specs::Builder;
        // let tex = GlTexture::from_bundle(
        //     textures.load_texture(&mut graphics.factory, "examples/block.png"),
        // );
        // let tex_rects = {
        //     let tex_rect = tex.source_rect();
        //     let back_rect = tex_rect.sub_rect([0, 0], [16, 16]);
        //     let front_rect = tex_rect.sub_rect([16, 0], [16, 16]);
        //     let left_rect = tex_rect.sub_rect([32, 0], [16, 16]);
        //     let right_rect = tex_rect.sub_rect([0, 16], [16, 16]);
        //     let bottom_rect = tex_rect.sub_rect([16, 16], [16, 16]);
        //     let top_rect = tex_rect.sub_rect([32, 16], [16, 16]);
        //     [
        //         back_rect,
        //         front_rect,
        //         left_rect,
        //         right_rect,
        //         bottom_rect,
        //         top_rect,
        //     ]
        // };
        // let _entity = world
        //     .create_entity()
        //     .with(
        //         MeshBuilder::new()
        //             // .quad(
        //             //     [0., 0., 0.],
        //             //     [1., 1.],
        //             //     // [colors::RED, colors::GREEN, colors::BLUE, colors::MAGENTA],
        //             //     [colors::WHITE, colors::WHITE, colors::WHITE, colors::WHITE],
        //             // )
        //             .pseudocube([0., 0., 0.], [1., 1., 1.], tex_rects)
        //             .build(&mut graphics),
        //     )
        //     .with(
        //         Transform::default()
        //             .with_anchor([0.5, 0.5, 0.5])
        //             .with_position([0.25, 0.25, 0.])
        //             .with_scale([0.5, 0.5, 0.5])
        //             .with_rotate_world(Deg(45.), Y_AXIS)
        //             .with_rotate_world(Deg(30.), X_AXIS),
        //     )
        //     .with(tex)
        //     .build();

        builder
    });

    app.run()?;

    Ok(())
}
