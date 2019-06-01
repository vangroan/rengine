extern crate rengine;

use rengine::angle::{Deg, Rad};
use rengine::camera::{ActiveCamera, CameraProjection, CameraView};
use rengine::comp::{GlTexture, Transform};
use rengine::glm;
use rengine::glutin::dpi::PhysicalPosition;
use rengine::nalgebra::{Matrix4, Perspective3, Point3, Unit, Vector3};
use rengine::option::lift2;
use rengine::res::{DeviceDimensions, TextureAssets};
use rengine::specs::{Builder, Entity, Read, ReadStorage, RunNow, World, Write, WriteStorage};
use rengine::voxel::{
    voxel_raycast, voxel_to_chunk, ChunkControl, ChunkCoord, ChunkMapping, ChunkUpkeepSystem,
    VoxelArrayChunk, VoxelBoxGen, VoxelChunk, VoxelCoord, VoxelData, VoxelRayInfo, VoxelRaycast,
    CHUNK_DIM8,
};
use rengine::{AppBuilder, Context, Scene, Trans};
use std::error::Error;

const BLOCK_TEX_PATH: &str = "examples/block.png";
type TileVoxelCtrl = ChunkControl<TileVoxel, VoxelArrayChunk<TileVoxel>>;
type TileVoxelChunk = VoxelArrayChunk<TileVoxel>;
type TileUpkeepSystem = ChunkUpkeepSystem<TileVoxel, TileVoxelChunk, VoxelBoxGen>;
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
        .with(tex)
        .with(TileVoxelChunk::new(chunk_id.clone()))
        .with(Transform::new().with_position([
            chunk_id.i as f32 * CHUNK_DIM8 as f32,
            chunk_id.j as f32 * CHUNK_DIM8 as f32,
            chunk_id.k as f32 * CHUNK_DIM8 as f32,
        ]))
        // .with(Transform::new().with_position([
        //     chunk_id.i as f32,
        //     chunk_id.j as f32,
        //     chunk_id.k as f32,
        // ]))
        .build();

    world
        .write_resource::<ChunkMapping>()
        .add_chunk(entity, chunk_id);

    entity
}

fn mouse_raycast(
    data: (
        Read<'_, ActiveCamera>,
        Read<'_, DeviceDimensions>,
        ReadStorage<'_, CameraView>,
        ReadStorage<'_, CameraProjection>,
    ),
    screen_pos: PhysicalPosition,
    steps: u32,
) -> Option<VoxelRaycast> {
    let (active_camera, device_dim, cam_views, cam_projs) = data;

    let maybe_cam = active_camera
        .camera_entity()
        .and_then(|e| lift2(cam_projs.get(e), cam_views.get(e)));

    if let Some((cam_proj, cam_view)) = maybe_cam {
        println!("# Casting Ray");

        println!("  Screen Position: {:?}", (screen_pos.x, screen_pos.y));

        // Build perspective projection that matches camera
        let projection = {
            let persp_settings = cam_proj.perspective_settings();
            Perspective3::new(
                persp_settings.aspect_ratio(),
                persp_settings.fovy().as_radians(),
                persp_settings.nearz(),
                persp_settings.farz(),
            )
        };

        // Get Camera World position
        let camera_pos = cam_view.position().clone();
        println!("  Camera Position: {}", camera_pos);

        // Point must be between [0.0, 1.0] to unproject
        let (device_w, device_h) = (
            device_dim.physical_size().width as f32,
            device_dim.physical_size().height as f32,
        );
        println!("  Device Dimensions: {:?}", (device_w, device_h));

        // Convert glutin screen position to computer graphics screen coordinates
        let (screen_w, screen_h) = (
            screen_pos.x as f32 - (device_w / 2.),
            -(screen_pos.y as f32 - (device_h / 2.)),
        );

        // Use screen position to compute two points in clip space, where near
        // and far are -1 and 1 respectively.
        //
        // "ndc" = normalized device coordinates
        let near_ndc_point = Point3::new(screen_w / device_w, screen_h / device_h, -1.0);
        let far_ndc_point = Point3::new(screen_w / device_w, screen_h / device_h, 1.0);
        println!("  Normalized Device Points:");
        println!("    Near: {}", near_ndc_point);
        println!("    Far: {}", far_ndc_point);

        // Unproject clip space points to view space
        let near_view_point = projection.unproject_point(&near_ndc_point);
        let far_view_point = projection.unproject_point(&far_ndc_point);
        println!("  View Space Points:");
        println!("    Near: {}", near_view_point);
        println!("    Far: {}", far_view_point);

        // Compute line in view space
        let line_point = near_view_point;
        let line_direction = Unit::new_normalize(far_view_point - near_view_point);
        println!("  Camera Local Line:");
        println!("    Point: {}", line_point);
        println!("    Direction: {}", line_direction.as_ref());

        // Transform line from local camera space to world space
        let inverse_view_mat = cam_view
            .view_matrix()
            .try_inverse()
            .expect("Failed to compute inverse of view matrix");

        // Inverse matrix to transform device space to world space
        let world_point = inverse_view_mat.transform_point(&line_point);
        let world_direction =
            Unit::new_normalize(inverse_view_mat.transform_vector(&line_direction));
        println!("  World Line:");
        println!("    Position: {}", world_point);
        println!("    Direction: {}", world_direction.as_ref());

        // Create ray walker
        return Some(voxel_raycast(world_point, world_direction, steps));
    }

    None
}

pub struct Game {
    chunk_upkeep_sys: Option<TileUpkeepSystem>,
    cursor_pos: PhysicalPosition,
    carve: bool,
    add: bool,
}

impl Game {
    fn new() -> Self {
        Game {
            chunk_upkeep_sys: None,
            cursor_pos: PhysicalPosition::new(0., 0.),
            carve: false,
            add: false,
        }
    }
}

impl Scene for Game {
    fn on_start(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        // Setup Voxels
        ctx.world.add_resource(TileVoxelCtrl::new());
        ctx.world.add_resource(ChunkMapping::new());
        ctx.world.register::<VoxelArrayChunk<TileVoxel>>();

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
        self.chunk_upkeep_sys = Some(TileUpkeepSystem::new(VoxelBoxGen::new(
            tex.clone(),
            tex_rects,
        )));

        // Create Chunks
        create_chunk(&mut ctx.world, ChunkCoord::new(0, 0, 0), tex.clone());
        create_chunk(&mut ctx.world, ChunkCoord::new(1, 0, 0), tex.clone());
        create_chunk(&mut ctx.world, ChunkCoord::new(0, 0, 1), tex.clone());
        create_chunk(&mut ctx.world, ChunkCoord::new(1, 0, 1), tex.clone());

        {
            let mapping = ctx.world.write_resource::<ChunkMapping>();
            let inner = mapping.inner();
            for kvp in inner.iter() {
                println!("{:?}", kvp);
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
        ctx.world.exec(
            |(active_camera, mut cam_views, mut _cam_projs): CameraData| {
                let pos = isometric_camera_position() * 70.;

                let maybe_cam = active_camera
                    .camera_entity()
                    .and_then(|e| lift2(_cam_projs.get_mut(e), cam_views.get_mut(e)));

                if let Some((_, view)) = maybe_cam {
                    view.set_position(pos);
                    view.look_at([0., 0., 0.].into());
                }
            },
        );

        None
    }

    fn on_event(&mut self, ctx: &mut Context<'_>, ev: &glutin::Event) -> Option<Trans> {
        use glutin::ElementState;
        use glutin::Event::*;
        use glutin::MouseButton;
        use glutin::WindowEvent::*;

        match ev {
            WindowEvent { event, .. } => match event {
                CursorMoved { position, .. } => {
                    let (device_dim,): (Read<'_, DeviceDimensions>,) = ctx.world.system_data();
                    self.cursor_pos = position.to_physical(device_dim.dpi_factor());
                }
                MouseInput { button, state, .. } => {
                    if button == &MouseButton::Right {
                        self.carve = state == &ElementState::Pressed;
                    } else if button == &MouseButton::Left {
                        self.add = state == &ElementState::Pressed;
                    }
                }
                _ => {}
            },
            _ => {}
        }

        None
    }

    fn on_update(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        if let Some(ref mut chunk_upkeep_sys) = self.chunk_upkeep_sys {
            chunk_upkeep_sys.run_now(&ctx.world.res);
        }

        if self.carve {
            if let Some(raycast) =
                mouse_raycast(ctx.world.system_data(), self.cursor_pos.clone(), 200)
            {
                let (chunk_map, mut chunk_ctrl, chunks): (
                    Read<'_, ChunkMapping>,
                    Write<'_, TileVoxelCtrl>,
                    ReadStorage<'_, VoxelArrayChunk<TileVoxel>>,
                ) = ctx.world.system_data();

                for raycast_info in raycast {
                    println!("Voxel: {}", raycast_info.voxel_coord());

                    // Determine chunk coordinate
                    let chunk_coord = voxel_to_chunk(raycast_info.voxel_coord());
                    let occupied = chunk_map
                        .chunk_entity(chunk_coord)
                        .and_then(|e| chunks.get(e))
                        .and_then(|c| c.get(raycast_info.voxel_coord().clone()))
                        .map(|d| d.occupied())
                        .unwrap_or(false);

                    // Carve out line in path of ray
                    if occupied {
                        println!("!! Carving {}", raycast_info.voxel_coord());
                        chunk_ctrl.lazy_update(
                            raycast_info.voxel_coord().clone(),
                            TileVoxel {
                                tile_id: EMPTY_TILE,
                            },
                        );
                    }
                }
            }
        }

        if self.add {
            if let Some(raycast) =
                mouse_raycast(ctx.world.system_data(), self.cursor_pos.clone(), 200)
            {
                let (chunk_map, mut chunk_ctrl, chunks): (
                    Read<'_, ChunkMapping>,
                    Write<'_, TileVoxelCtrl>,
                    ReadStorage<'_, VoxelArrayChunk<TileVoxel>>,
                ) = ctx.world.system_data();

                let mut last_voxel = VoxelCoord::new(9999, 9999, 9999);

                'cast: for raycast_info in raycast {
                    println!("Voxel: {}", raycast_info.voxel_coord());

                    // Determine chunk coordinate
                    let chunk_coord = voxel_to_chunk(raycast_info.voxel_coord());
                    let occupied = chunk_map
                        .chunk_entity(chunk_coord)
                        .and_then(|e| chunks.get(e))
                        .and_then(|c| c.get(raycast_info.voxel_coord().clone()))
                        .map(|d| d.occupied())
                        .unwrap_or(false);

                    // Tile hit, add to previous
                    if occupied {
                        println!("!! Adding {}", last_voxel);
                        chunk_ctrl.lazy_update(last_voxel.clone(), TileVoxel { tile_id: 1 });

                        // Stop
                        break 'cast;
                    } else {
                        last_voxel = raycast_info.voxel_coord().clone();
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
        .size(500, 500)
        .background_color([0.3, 0.4, 0.5, 1.0])
        .init_scene(Game::new())
        .build()?;

    app.run()?;

    Ok(())
}
