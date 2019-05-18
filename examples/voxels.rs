extern crate rengine;

use rengine::camera::{ActiveCamera, CameraProjection, CameraView};
use rengine::comp::{GlTexture, Transform};
use rengine::nalgebra::Point3;
use rengine::option::lift2;
use rengine::res::{AssetBundle, TextureAssets};
use rengine::specs::{Builder, Entity, Read, ReadStorage, RunNow, World, Write, WriteStorage};
use rengine::voxel::{
    ChunkControl, ChunkCoord, ChunkMapping, ChunkUpkeepSystem, VoxelArrayChunk, VoxelBoxGen,
    VoxelData, CHUNK_DIM8,
};
use rengine::{AppBuilder, Context, Scene, Trans};
use std::error::Error;
use std::sync::Arc;

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

fn create_chunk(world: &mut World, chunk_id: ChunkCoord, tex_bundle: Arc<AssetBundle>) -> Entity {
    let entity = world
        .create_entity()
        .with(GlTexture::from_bundle(tex_bundle))
        .with(TileVoxelChunk::new(chunk_id.clone()))
        .with(Transform::new())
        .build();

    world
        .write_resource::<ChunkMapping>()
        .add_chunk(entity, chunk_id);

    entity
}

pub struct Game {
    chunk_upkeep_sys: Option<TileUpkeepSystem>,
    voxel_tex: Option<GlTexture>,
}

impl Game {
    fn new() -> Self {
        Game {
            chunk_upkeep_sys: None,
            voxel_tex: None,
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
        let tex_bundle = ctx
            .world
            .write_resource::<TextureAssets>()
            .default_texture(ctx.graphics.factory_mut());

        // Setup system
        self.chunk_upkeep_sys = Some(TileUpkeepSystem::new(VoxelBoxGen::new(
            GlTexture::from_bundle(tex_bundle.clone()),
        )));

        // Create Chunks
        let e = create_chunk(&mut ctx.world, ChunkCoord::new(0, 0, 0), tex_bundle);

        // Fill chunk with some data
        ctx.world.exec(|(mut ctrl,): (Write<'_, TileVoxelCtrl>,)| {
            for x in 0..CHUNK_DIM8 {
                for y in 0..CHUNK_DIM8 {
                    for z in 0..CHUNK_DIM8 {
                        ctrl.lazy_update([x as i32, y as i32, z as i32], TileVoxel { tile_id: 1 });
                    }
                }
            }
        });

        // Position Camera
        ctx.world.exec(
            |(active_camera, mut cam_views, mut _cam_projs): CameraData| {
                let pos = Point3::new(0.2, 0.1, 5.);

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
