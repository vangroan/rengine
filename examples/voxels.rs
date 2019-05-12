extern crate rengine;

use rengine::comp::GlTexture;
use rengine::res::{AssetBundle, TextureAssets};
use rengine::specs::{Builder, Entity, Read, ReadStorage, RunNow, World, Write, WriteStorage};
use rengine::voxel::{
    ChunkControl, ChunkCoord, ChunkMapping, ChunkUpkeepSystem, VoxelArrayChunk, VoxelBoxGen,
    VoxelData,
};
use rengine::{AppBuilder, Context, Scene, Trans};
use std::error::Error;
use std::sync::Arc;

type TileVoxelCtrl = ChunkControl<TileVoxel, VoxelArrayChunk<TileVoxel>>;
type TileVoxelChunk = VoxelArrayChunk<TileVoxel>;
type TileUpkeepSystem = ChunkUpkeepSystem<TileVoxel, TileVoxelChunk, VoxelBoxGen>;
const EMPTY_TILE: u16 = 0;

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
        .build();

    world
        .write_resource::<ChunkMapping>()
        .add_chunk(entity, chunk_id);

    entity
}

pub struct Game {
    chunk_upkeep_sys: TileUpkeepSystem,
    voxel_tex: Option<GlTexture>,
}

impl Game {
    fn new() -> Self {
        Game {
            chunk_upkeep_sys: TileUpkeepSystem::new(),
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

        // Create Chunks
        create_chunk(&mut ctx.world, ChunkCoord::new(0, 0, 0), tex_bundle);

        None
    }

    fn on_update(&mut self, ctx: &mut Context<'_>) -> Option<Trans> {
        self.chunk_upkeep_sys.run_now(&ctx.world.res);

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
