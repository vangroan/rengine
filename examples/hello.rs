extern crate rengine;
#[macro_use]
extern crate specs_derive;

use rengine::angle::Deg;
use rengine::camera::{ActiveCamera, CameraProjection, CameraView};
use rengine::comp::{GlTexture, MeshBuilder, Transform, X_AXIS, Y_AXIS};
use rengine::glutin::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use rengine::nalgebra::Vector3;
use rengine::option::lift2;
use rengine::res::{DeltaTime, TextureAssets};
use rengine::specs::{
    Builder, Component, DenseVecStorage, Entity, Join, Read, ReadExpect, ReadStorage, WriteStorage,
};
use rengine::{Context, GlTextureAssets, Scene, Trans};
use std::error::Error;
use std::fmt;

const BLOCK_TEX_PATH: &str = "examples/block.png";

type CameraData<'a> = (
    ReadExpect<'a, ActiveCamera>,
    WriteStorage<'a, CameraView>,
    WriteStorage<'a, CameraProjection>,
);

#[derive(Component)]
pub struct Block;

#[derive(Debug)]
struct Intro;

impl Scene for Intro {
    fn on_start(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        println!("{:?}: On start", self);

        ctx.world.register::<Block>();

        Trans::replace(Game::default())
    }

    fn on_stop(&mut self, _ctx: &mut Context<'_>) -> Option<Trans> {
        println!("{:?}: On stop", self);

        None
    }
}

#[derive(Debug)]
struct Game {
    // Camera move speed, unit per second
    camera_speed: f32,

    // Intended direction of camera movement
    camera_dir: Vector3<f32>,

    entities: Vec<Entity>,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            camera_speed: 10.,
            camera_dir: Vector3::new(0., 0., -2.),
            entities: Vec::new(),
        }
    }
}

impl Scene for Game {
    fn on_start(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        println!("{}: On start", self);

        // Position camera away from cubes
        ctx.world.exec(
            |(active_camera, mut cam_views, mut _cam_projs): CameraData| {
                let maybe_cam = active_camera
                    .camera_entity()
                    .and_then(|e| lift2(_cam_projs.get_mut(e), cam_views.get_mut(e)));

                if let Some((_, view)) = maybe_cam {
                    view.set_position([0., 0., 0.].into());
                    view.look_at([0., 0., -1.].into());
                }
            },
        );

        // Test Pseudocube
        let tex = GlTexture::from_bundle(
            ctx.world
                .write_resource::<GlTextureAssets>()
                .load_texture(&mut ctx.graphics.factory_mut(), BLOCK_TEX_PATH),
        );
        let tex_rects = {
            let tex_rect = tex.source_rect();
            let back_rect = tex_rect.sub_rect([0, 0], [16, 16]);
            let front_rect = tex_rect.sub_rect([16, 0], [16, 16]);
            let left_rect = tex_rect.sub_rect([32, 0], [16, 16]);
            let right_rect = tex_rect.sub_rect([0, 16], [16, 16]);
            let bottom_rect = tex_rect.sub_rect([16, 16], [16, 16]);
            let top_rect = tex_rect.sub_rect([32, 16], [16, 16]);
            [
                back_rect,
                front_rect,
                left_rect,
                right_rect,
                bottom_rect,
                top_rect,
            ]
        };
        let entity = ctx
            .world
            .create_entity()
            .with(
                MeshBuilder::new()
                    // .quad(
                    //     [0., 0., 0.],
                    //     [1., 1.],
                    //     // [colors::RED, colors::GREEN, colors::BLUE, colors::MAGENTA],
                    //     [colors::WHITE, colors::WHITE, colors::WHITE, colors::WHITE],
                    // )
                    .pseudocube([0., 0., 0.], [0.5, 0.5, 0.5], tex_rects)
                    .build(&mut ctx.graphics),
            )
            .with(
                Transform::default()
                    .with_anchor([0.0, 0.0, 0.0])
                    .with_position([0.0, 0.0, 0.0])
                    // .with_scale([0.5, 0.5, 0.5])
                    // .with_rotate_world(Deg(45.), Y_AXIS)
                    // .with_rotate_world(Deg(30.), X_AXIS),
            )
            .with(tex)
            .build();

        self.entities.push(entity);

        None
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        println!("{}: On stop", self);

        if let Err(err) = ctx.world.delete_entities(&self.entities) {
            panic!(err);
        }

        ctx.world
            .write_resource::<TextureAssets>()
            .remove_texture(BLOCK_TEX_PATH);

        None
    }

    fn on_update(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        let dt = {
            let (delta_time,): (Read<DeltaTime>,) = ctx.world.system_data();
            delta_time.as_secs_float()
        };

        // Camera
        {
            let (active_camera, mut cam_views, mut _cam_projs): CameraData =
                ctx.world.system_data();

            let maybe_cam = active_camera
                .camera_entity()
                .and_then(|e| lift2(_cam_projs.get_mut(e), cam_views.get_mut(e)));

            if let Some((_proj, view)) = maybe_cam {
                let translate = self.camera_dir * self.camera_speed * dt;
                let pos = view.position();
                view.set_position(pos + translate);
                let target = view.target();
                view.look_at(target + translate);

                {
                    let pos = view.position();
                    let target = view.target();
                    println!(
                        "Camera Pos ({}, {}, {}); Target ({}, {}, {})",
                        pos.x, pos.y, pos.z, target.x, target.y, target.z
                    );
                }
            }
        }

        // Block
        ctx.world.exec(
            |(_blocks, mut transforms): (ReadStorage<Block>, WriteStorage<Transform>)| {
                for (ref mut transform,) in (&mut transforms,).join() {
                    // let translate = Vector3::new(0.0, 0.1, 0.0) * dt;
                    // transform.translate(translate);
                }
            },
        );

        // Clear direction for next frame
        self.camera_dir = Vector3::new(0., 0., 0.);

        None
    }

    fn on_event(&mut self, _ctx: &mut Context<'_>, event: &Event) -> Option<Trans> {
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(key),
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => match key {
                    VirtualKeyCode::W => {
                        self.camera_dir.y = 1.;
                    }
                    VirtualKeyCode::S => {
                        self.camera_dir.y = -1.;
                    }
                    VirtualKeyCode::A => {
                        self.camera_dir.x = -1.;
                    }
                    VirtualKeyCode::D => {
                        self.camera_dir.x = 1.;
                    }
                    VirtualKeyCode::F => {
                        self.camera_dir.z = 1.;
                    }
                    VirtualKeyCode::R => {
                        self.camera_dir.z = -1.;
                    }
                    _ => {}
                },
                ElementState::Released => {}
            },
            _ => {}
        }

        None
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Game")
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
