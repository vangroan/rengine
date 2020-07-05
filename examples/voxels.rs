extern crate rengine;

use std::{borrow::Cow, error::Error};

use log::trace;
use rengine::angle::{Deg, Rad};
use rengine::camera::{
    ActiveCamera, CameraDriftSystem, CameraProjection, CameraView, DollyCamera,
    DollyCameraControlSystem, FocusTarget, GridCamera, GridCameraControlSystem, OrbitalCamera,
    OrbitalCameraControlSystem, SlideCamera, SlideCameraControlSystem,
};
use rengine::colors::WHITE;
use rengine::comp::{GlTexture, MeshBuilder, Transform};
use rengine::glm;
use rengine::glutin::dpi::PhysicalPosition;
use rengine::gui::text::TextBatch;
use rengine::metrics::{builtin_metrics::*, DataPoint, MetricAggregate, MetricHub};
use rengine::modding::{Mods, SceneHook, ScriptChannel};
use rengine::nalgebra::{Point3, Vector3};
use rengine::option::lift2;
use rengine::render::{Gizmo, Material};
use rengine::res::{DeltaTime, DeviceDimensions, TextureAssets};
use rengine::rlua::{UserData, UserDataMethods};
use rengine::scripting;
use rengine::scripting::prelude::*;
use rengine::specs::prelude::*;
use rengine::sprite::{Billboard, BillboardSystem};
use rengine::util::FpsCounter;
use rengine::voxel::{
    raycast_from_camera, voxel_to_chunk, ChunkControl, ChunkCoord, ChunkMapping, ChunkUpkeepSystem,
    DeformedBoxGen, VoxelArrayChunk, VoxelChunk, VoxelCoord, VoxelData, CHUNK_DIM8,
};
use rengine::{AppBuilder, Context, GraphicContext, Scene, Trans};
use serde::Deserialize;

const BLOCK_TEX_PATH: &str = "examples/block.png";
type TileVoxelCtrl = ChunkControl<TileVoxel, VoxelArrayChunk<TileVoxel>>;
type TileVoxelChunk = VoxelArrayChunk<TileVoxel>;
type TileUpkeepSystem = ChunkUpkeepSystem<TileVoxel, TileVoxelChunk, DeformedBoxGen>;
const EMPTY_TILE: u16 = 0;
type CameraData<'a> = (
    Read<'a, ActiveCamera>,
    WriteStorage<'a, CameraView>,
    WriteStorage<'a, CameraProjection>,
);

#[derive(Copy, Clone, Default)]
struct TileVoxel {
    tile_id: u16,
}

impl VoxelData for TileVoxel {
    #[inline]
    fn occupied(&self) -> bool {
        self.tile_id != EMPTY_TILE
    }
}

#[allow(clippy::just_underscores_and_digits)]
fn isometric_camera_position() -> Point3<f32> {
    let _45 = Deg(45.);
    let _35 = Rad((1. / 2.0_f32.sqrt()).atan());

    let p = Point3::new(0., 0., 1.);

    let rot_45 = glm::quat_angle_axis(_45.as_radians(), &Vector3::y_axis());
    let rot_35 = glm::quat_angle_axis(-_35.as_radians(), &Vector3::x_axis());

    let m = glm::quat_to_mat4(&rot_45) * glm::quat_to_mat4(&rot_35);

    m.transform_point(&p)
}

fn create_chunk(world: &mut World, chunk_id: ChunkCoord, tex: GlTexture) -> Entity {
    // Note: Mesh is generated later
    let entity = world
        .create_entity()
        .with(Material::Basic { texture: tex })
        .with(Gizmo)
        .with(TileVoxelChunk::new(chunk_id.clone()))
        .with(Transform::new().with_position([
            chunk_id.i as f32 * CHUNK_DIM8 as f32,
            chunk_id.j as f32 * CHUNK_DIM8 as f32,
            chunk_id.k as f32 * CHUNK_DIM8 as f32,
        ]))
        .build();

    world
        .write_resource::<ChunkMapping>()
        .add_chunk(entity, chunk_id);

    entity
}

fn create_sprite<V: Into<glm::Vec3>>(
    world: &mut World,
    graphics: &mut GraphicContext,
    pos: V,
    tex: GlTexture,
) -> Entity {
    world
        .create_entity()
        .with(Material::Basic { texture: tex })
        .with(Gizmo)
        .with(Billboard)
        .with(
            MeshBuilder::new()
                .quad_with_uvs(
                    [0.0, 0.0, 0.0],
                    [1.0, 1.0],
                    [WHITE, WHITE, WHITE, WHITE],
                    [[0.0, 0.25], [0.25, 0.25], [0.25, 0.0], [0.0, 0.0]],
                )
                .build(graphics),
        )
        .with(Transform::default().with_position(pos))
        .build()
}

fn create_script_api(lua: &mut rengine::rlua::Lua, script_channel: ScriptChannel) {
    let _result: rlua::Result<()> = lua.context(|ctx| {
        let sender = script_channel.clone();
        let lib = ctx.create_table()?;

        let spawn_skelly = ctx.create_function(
            move |_, (x, y, z): (rlua::Number, rlua::Number, rlua::Number)| {
                // let mut sender = script_channel.clone();
                println!("spawn_skelly({}, {}, {})", x, y, z);

                // sender.send(x as u32);
                sender.clone().send(x as u32);

                Ok(())
            },
        )?;
        lib.set("spawn_skelly", spawn_skelly)?;

        let globals = ctx.globals();
        globals.set("skelly", lib)?;
        Ok(())
    });
}

fn handle_script_commands(_world: &World, cmds: &[u32]) {
    for cmd in cmds {
        println!("handle script commands {}", cmd);
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct ExamplePrototype {
    name: String,
}

impl Prototype for ExamplePrototype {
    fn type_name<'a>() -> Cow<'a, str> {
        "example".into()
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct SoldierPrototype {
    name: String,
    descriptive: String,
    position: [f32; 3],
    texture_path: String,
}

impl Prototype for SoldierPrototype {
    fn type_name<'a>() -> Cow<'a, str> {
        "soldier".into()
    }
}

fn test_load() -> rlua::Result<()> {
    use rlua::Lua;
    use rlua_serde;

    let mut lua = Lua::new();
    let result: rlua::Result<()> = lua.context(|lua_ctx| {
        // let mut skelly_proto = SkellyPrototype::default();

        let skelly_val = lua_ctx
            .load(
                r#"
                {
                    name = 'skelly_soldier',
                    descriptive = 'Skeleton Soldier',
                    position = { 0.0, 0.0, 0.0 },
                    texture_path = 'examples/skelly.png',
                }
                "#,
            )
            .eval::<rlua::Value>()?;

        let skelly_proto: SoldierPrototype = rlua_serde::from_value(skelly_val).unwrap();
        println!("{:?}", skelly_proto);

        Ok(())
    });
    result?;

    Ok(())
}

struct LuaWorld<'a> {
    entities: &'a mut Vec<Entity>,
    world: &'a mut World,
    graphics: &'a mut GraphicContext,
    prototypes: &'a PrototypeTable,
}

impl<'a> UserData for LuaWorld<'a> {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method_mut(
            "spawn_soldier",
            |_lua_ctx, lua_world, (proto_name, params): (String, rlua::Table)| {
                use rlua::Value;
                // create_sprite
                println!("spawn skelly");
                let proto = lua_world
                    .prototypes
                    .get::<SoldierPrototype>(proto_name.as_str())
                    .unwrap_or_else(|| panic!("No prototype registered called '{}'", proto_name));

                let skelly_tex = GlTexture::from_bundle(
                    lua_world
                        .world
                        .write_resource::<TextureAssets>()
                        .load_texture(
                            &mut lua_world.graphics.factory_mut(),
                            proto.texture_path.as_str(),
                        ),
                );

                let param_pos: Value = params.get("position")?;
                let pos: [f32; 3] = if let Value::Table(_) = param_pos {
                    rlua_serde::from_value(param_pos).unwrap()
                } else {
                    proto.position
                };

                let entity = create_sprite(
                    &mut lua_world.world,
                    &mut lua_world.graphics,
                    pos,
                    skelly_tex,
                );

                lua_world.entities.push(entity);

                Ok(())
            },
        );
    }
}

// ----- //
// Scene //
// ----- //

pub struct Game {
    mods: scripting::Mods,
    chunk_upkeep_sys: Option<TileUpkeepSystem>,
    billboard_sys: BillboardSystem,
    orbital_sys: OrbitalCameraControlSystem,
    dolly_sys: DollyCameraControlSystem,
    grid_camera_sys: GridCameraControlSystem,
    slide_camera_sys: SlideCameraControlSystem,
    camera_drift_sys: CameraDriftSystem,
    cursor_pos: PhysicalPosition,
    carve: bool,
    carved: bool,
    add: bool,
    added: bool,
    entities: Vec<Entity>,
}

impl Game {
    fn new() -> Self {
        Game {
            mods: scripting::Mods::from_path("./examples/mods").unwrap(),
            chunk_upkeep_sys: None,
            billboard_sys: BillboardSystem,
            orbital_sys: OrbitalCameraControlSystem::new(),
            dolly_sys: DollyCameraControlSystem::new(),
            grid_camera_sys: GridCameraControlSystem::new(),
            slide_camera_sys: SlideCameraControlSystem::new(),
            camera_drift_sys: CameraDriftSystem::new(),
            cursor_pos: PhysicalPosition::new(0., 0.),
            carve: false,
            carved: false,
            add: false,
            added: false,
            entities: vec![],
        }
    }
}

impl Scene for Game {
    fn on_start(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        test_load().unwrap();

        // Setup Voxels
        ctx.world.add_resource(TileVoxelCtrl::new());
        ctx.world.add_resource(ChunkMapping::new());
        ctx.world.register::<VoxelArrayChunk<TileVoxel>>();
        ctx.world.register::<Billboard>();

        // Load Texture
        let tex = GlTexture::from_bundle(
            ctx.world
                .write_resource::<TextureAssets>()
                .load_texture(&mut ctx.graphics.factory_mut(), BLOCK_TEX_PATH),
        );

        // Block Texture
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

        // Setup system
        self.chunk_upkeep_sys = Some(TileUpkeepSystem::new(DeformedBoxGen::new(0.1, tex_rects)));

        // Create Chunks
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    self.entities.push(create_chunk(
                        &mut ctx.world,
                        ChunkCoord::new(x, y, z),
                        tex.clone(),
                    ));
                }
            }
        }

        {
            let mapping = ctx.world.write_resource::<ChunkMapping>();
            let inner = mapping.inner();
            for kvp in inner.iter() {
                trace!("{:?}", kvp);
            }
        }

        // Fill chunk with some data
        let size2 = CHUNK_DIM8 * 2;
        ctx.world.exec(|(mut ctrl,): (Write<'_, TileVoxelCtrl>,)| {
            for x in 0..size2 {
                for y in 0..CHUNK_DIM8 {
                    for z in 0..size2 {
                        ctrl.lazy_update([x as i32, y as i32, z as i32], TileVoxel { tile_id: 1 });
                    }
                }
            }
        });

        // Position Camera
        let device_dim = DeviceDimensions::from_window(ctx.graphics.window()).unwrap();
        let logical_dim: (u32, u32) = device_dim.logical_size().clone().into();
        let camera_id = ctx
            .world
            .create_entity()
            .with(Transform::new().with_position([0., 0., -2.]))
            .with(CameraProjection::with_device_size((
                logical_dim.0 as u16,
                logical_dim.1 as u16,
            )))
            .with(CameraView::new())
            .with(FocusTarget::with_target([
                CHUNK_DIM8 as f32,
                CHUNK_DIM8 as f32,
                CHUNK_DIM8 as f32,
            ]))
            .with(OrbitalCamera::new())
            .with(DollyCamera::new())
            .with(GridCamera::new())
            .with(SlideCamera::new())
            .build();
        ctx.world
            .write_resource::<ActiveCamera>()
            .set_camera_entity(camera_id);
        self.entities.push(camera_id);

        ctx.world.exec(
            |(active_camera, mut cam_views, mut _cam_projs): CameraData| {
                let pos = isometric_camera_position() * 70.;

                let maybe_cam = active_camera
                    .camera_entity()
                    .and_then(|e| lift2(_cam_projs.get_mut(e), cam_views.get_mut(e)));

                if let Some((_, view)) = maybe_cam {
                    view.set_position(pos);
                    view.look_at([CHUNK_DIM8 as f32, CHUNK_DIM8 as f32, CHUNK_DIM8 as f32].into());
                }
            },
        );

        // Create Sprites
        let _default_texture = GlTexture::from_bundle(
            ctx.world
                .write_resource::<TextureAssets>()
                .default_texture(&mut ctx.graphics.factory_mut()),
        );
        let skelly_tex = GlTexture::from_bundle(
            ctx.world
                .write_resource::<TextureAssets>()
                .load_texture(&mut ctx.graphics.factory_mut(), "examples/skelly.png"),
        );

        for x in 1..5 {
            for z in 1..5 {
                self.entities.push(create_sprite(
                    &mut ctx.world,
                    &mut ctx.graphics,
                    [x as f32 * 2.5, 8.0 + 0.5, z as f32 * 2.5],
                    skelly_tex.clone(),
                ));
            }
        }

        // FPS Counter
        self.entities
            .push(rengine::util::create_fps_counter_widget(&mut ctx.world));

        // Load Mod Meta
        ctx.world.exec(|mut mods: WriteExpect<Mods>| {
            if let Err(e) = mods.load_mods() {
                println!("{:?}", e);
            }
            if let Err(e) = mods.init_mods(create_script_api) {
                println!("{:?}", e);
            }
        });

        self.mods.register_prototype::<ExamplePrototype>();
        self.mods.register_prototype::<SoldierPrototype>();

        self.mods
            .load_mods()
            .expect("game state error loading mods");
        self.mods
            .data_stage()
            .expect("game state error during data stage");

        {
            let entities = &mut self.entities;
            let prototypes = &self.mods.prototypes();
            for mod_bundle in self.mods.iter() {
                let result: rlua::Result<()> = mod_bundle.lua.context(|lua_ctx| {
                    lua_ctx.scope(|scope| {
                        use std::fs::{canonicalize, File};
                        use std::io::prelude::*;
                        use walkdir::WalkDir;

                        let lua_world = LuaWorld {
                            entities,
                            world: ctx.world,
                            graphics: ctx.graphics,
                            prototypes,
                        };
                        let world_user_data = scope.create_nonstatic_userdata(lua_world)?;
                        let globals = lua_ctx.globals();
                        globals.set("GAME", world_user_data)?;

                        let mut buf = vec![];
                        let walker = WalkDir::new(mod_bundle.meta.path());
                        for entry in walker {
                            let entry = entry.unwrap();
                            let file_path = canonicalize(entry.path()).unwrap();

                            if entry.path().file_name().unwrap() == "init.lua" {
                                if !file_path.is_file() {
                                    continue;
                                }

                                let mut file = File::open(&file_path).unwrap();
                                buf.clear();
                                file.read_to_end(&mut buf).unwrap();

                                lua_ctx.load(&buf).exec()?;
                                if let Ok(func) = globals.get::<_, rlua::Function>("on_init") {
                                    func.call(())?;
                                }

                                continue;
                            }
                        }

                        Ok(())
                    })
                });
                result.unwrap();
            }
        }

        // Buttons
        {
            use rengine::gui::{widgets, WidgetBuilder};
            let (btn_group_entity, btn_group_node_id) = widgets::Container::vbox()
                .with_margin([8.0, 8.0])
                .with_placement([0.0, 16.0])
                .build(&mut ctx.world, &mut ctx.graphics);
            self.entities.push(btn_group_entity);

            let (btn_entity, _btn_id) = widgets::Button::text("Brush")
                .child_of(btn_group_node_id)
                .size(64., 64.)
                .background_image("examples/ui.png")
                .background_src_rect([0, 0], [32, 32])
                .build(&mut ctx.world, &mut ctx.graphics);
            self.entities.push(btn_entity);
        }

        // Execute mod start.
        //
        // In a real game, the mod load, init and start can happen
        // in different scenes at different times.
        let cmds =
            ctx.world.exec(
                |mut mods: WriteExpect<Mods>| match mods.scene_hook(SceneHook::Start) {
                    Ok(out_cmds) => out_cmds.unwrap_or_else(Vec::new),
                    Err(e) => {
                        println!("{:?}", e);
                        vec![]
                    }
                },
            );
        handle_script_commands(&ctx.world, &cmds);

        None
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        // Notify scripts that scene will stop.
        let cmds =
            ctx.world.exec(
                |mut mods: WriteExpect<Mods>| match mods.scene_hook(SceneHook::Stop) {
                    Ok(out_cmds) => out_cmds.unwrap_or_else(Vec::new),
                    Err(e) => {
                        println!("{:?}", e);
                        vec![]
                    }
                },
            );
        handle_script_commands(&ctx.world, &cmds);

        // Shutdown mods
        ctx.world.exec(|mut mods: WriteExpect<Mods>| {
            mods.shutdown().expect("mod shutdown");
        });

        // ensure entities are freed
        let maybe_err = ctx.world.delete_entities(&self.entities).err();
        if let Some(err) = maybe_err {
            panic!(err);
        }
        self.entities.clear();

        // Clear unused resources
        ctx.world
            .write_resource::<TextureAssets>()
            .remove_texture(BLOCK_TEX_PATH);

        None
    }

    fn on_event(&mut self, ctx: &mut Context<'_>, ev: &glutin::Event) -> Option<Trans> {
        use glutin::ElementState;
        use glutin::Event::*;
        use glutin::MouseButton;
        use glutin::VirtualKeyCode;
        use glutin::WindowEvent::*;

        rengine::gui::GuiLayoutSystem.run_now(&ctx.world.res);

        if let WindowEvent { event, .. } = ev {
            match event {
                CursorMoved { position, .. } => {
                    let (device_dim,): (Read<'_, DeviceDimensions>,) = ctx.world.system_data();
                    self.cursor_pos = position.to_physical(device_dim.dpi_factor());
                }
                MouseInput { button, state, .. } => {
                    if button == &MouseButton::Right {
                        self.carve = state == &ElementState::Pressed && !self.carved;

                        if state == &ElementState::Released {
                            self.carved = false;
                        }
                    } else if button == &MouseButton::Left {
                        self.add = state == &ElementState::Pressed && !self.added;

                        if state == &ElementState::Released {
                            self.added = false;
                        }
                    }
                }
                KeyboardInput { input, .. } => {
                    if input.virtual_keycode == Some(VirtualKeyCode::F5)
                        && input.state == ElementState::Released
                    {
                        ctx.world.exec(|metrics: Read<'_, MetricHub>| {
                            let length = 64;
                            let mut timeseries = vec![DataPoint::default(); length];

                            let now_seconds = chrono::Local::now().timestamp();

                            // -------------------------------------------------
                            println!("Render Time Taken");
                            metrics.make_time_series(
                                GRAPHICS_RENDER,
                                MetricAggregate::Maximum,
                                &mut timeseries,
                                0,
                                length,
                            );
                            let mut data_points: Vec<(i64, f64)> = timeseries
                                .iter()
                                .map(|dp| {
                                    let delta_seconds = now_seconds - dp.datetime.timestamp();
                                    (delta_seconds, dp.value)
                                })
                                .collect();
                            data_points.sort_by(|a, b| a.0.cmp(&b.0));

                            for dp in &data_points {
                                println!("{}: {}ms", dp.0, dp.1);
                            }

                            // Reset for next metric
                            for e in &mut timeseries {
                                *e = Default::default();
                            }

                            // -------------------------------------------------
                            println!("Draw Call Count");
                            metrics.make_time_series(
                                GRAPHICS_DRAW_CALLS,
                                MetricAggregate::Sum,
                                &mut timeseries,
                                0,
                                length,
                            );
                            let mut data_points: Vec<(i64, f64)> = timeseries
                                .iter()
                                .map(|dp| {
                                    let delta_seconds = now_seconds - dp.datetime.timestamp();
                                    (delta_seconds, dp.value)
                                })
                                .collect();
                            data_points.sort_by(|a, b| a.0.cmp(&b.0));

                            for dp in &data_points {
                                println!("{}: {}", dp.0, dp.1);
                            }
                        });
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn on_update(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        ctx.world.exec(
            |(dt, mut text_batches, mut fps_counters): (
                Read<DeltaTime>,
                WriteStorage<'_, TextBatch>,
                WriteStorage<'_, FpsCounter>,
            )| {
                for (text, counter) in (&mut text_batches, &mut fps_counters).join() {
                    counter.add(dt.duration());
                    text.replace(&format!("FPS: {:.2}", counter.fps()), WHITE);
                }
            },
        );

        self.orbital_sys.run_now(&ctx.world.res);
        self.dolly_sys.run_now(&ctx.world.res);
        self.grid_camera_sys.run_now(&ctx.world.res);
        self.slide_camera_sys.run_now(&ctx.world.res);
        self.camera_drift_sys.run_now(&ctx.world.res);

        if let Some(ref mut chunk_upkeep_sys) = self.chunk_upkeep_sys {
            chunk_upkeep_sys.run_now(&ctx.world.res);
        }

        // Orient sprites toward camera
        self.billboard_sys.run_now(&ctx.world.res);

        if self.carve && !self.carved {
            if let Some(raycast) =
                raycast_from_camera(ctx.world.system_data(), self.cursor_pos, 200)
            {
                let (chunk_map, mut chunk_ctrl, chunks): (
                    Read<'_, ChunkMapping>,
                    Write<'_, TileVoxelCtrl>,
                    ReadStorage<'_, VoxelArrayChunk<TileVoxel>>,
                ) = ctx.world.system_data();

                'carve: for raycast_info in raycast {
                    // Determine chunk coordinate
                    let chunk_coord = voxel_to_chunk(raycast_info.voxel_coord());
                    let occupied = chunk_map
                        .chunk_entity(chunk_coord)
                        .and_then(|e| chunks.get(e))
                        .and_then(|c| c.get(*raycast_info.voxel_coord()))
                        .map(|d| d.occupied())
                        .unwrap_or(false);

                    // Carve out a voxel in path of ray
                    if occupied {
                        chunk_ctrl.lazy_update(
                            *raycast_info.voxel_coord(),
                            TileVoxel {
                                tile_id: EMPTY_TILE,
                            },
                        );
                        self.carved = true;
                        break 'carve;
                    }
                }
            }
        }

        if self.add && !self.added {
            if let Some(raycast) =
                raycast_from_camera(ctx.world.system_data(), self.cursor_pos, 200)
            {
                let (chunk_map, mut chunk_ctrl, chunks): (
                    Read<'_, ChunkMapping>,
                    Write<'_, TileVoxelCtrl>,
                    ReadStorage<'_, VoxelArrayChunk<TileVoxel>>,
                ) = ctx.world.system_data();

                let mut last_voxel: Option<VoxelCoord> = None;

                'add: for raycast_info in raycast {
                    // Determine chunk coordinate
                    let chunk_coord = voxel_to_chunk(raycast_info.voxel_coord());
                    let occupied = chunk_map
                        .chunk_entity(chunk_coord)
                        .and_then(|e| chunks.get(e))
                        .and_then(|c| c.get(*raycast_info.voxel_coord()))
                        .map(|d| d.occupied())
                        .unwrap_or(false);

                    // Tile hit, add to previous
                    if occupied {
                        if let Some(last_voxel) = last_voxel {
                            chunk_ctrl.lazy_update(last_voxel, TileVoxel { tile_id: 1 });

                            self.added = true;
                        }

                        // Stop
                        break 'add;
                    } else {
                        last_voxel = Some(*raycast_info.voxel_coord());
                    }
                }
            }
        }

        None
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = AppBuilder::new()
        .title("Voxels Example")
        .size(800, 600)
        .background_color([0.3, 0.4, 0.5, 1.0])
        .init_scene(Game::new())
        .add_modding("rengine", "./examples/mods")
        .build()?;

    app.run()?;

    Ok(())
}
