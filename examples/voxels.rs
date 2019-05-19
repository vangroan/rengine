extern crate rengine;

use rengine::angle::{Deg, Rad};
use rengine::camera::{ActiveCamera, CameraProjection, CameraView};
use rengine::comp::{GlTexture, Transform};
use rengine::glm;
use rengine::nalgebra::{Point3, Vector3};
use rengine::option::lift2;
use rengine::res::TextureAssets;
use rengine::specs::{Builder, Entity, Read, RunNow, World, Write, WriteStorage};
use rengine::voxel::{
    ChunkControl, ChunkCoord, ChunkMapping, ChunkUpkeepSystem, VoxelArrayChunk, VoxelBoxGen,
    VoxelData, CHUNK_DIM8,
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

pub struct Game {
    chunk_upkeep_sys: Option<TileUpkeepSystem>,
}

impl Game {
    fn new() -> Self {
        Game {
            chunk_upkeep_sys: None,
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

    fn on_update(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        if let Some(ref mut chunk_upkeep_sys) = self.chunk_upkeep_sys {
            chunk_upkeep_sys.run_now(&ctx.world.res);
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
