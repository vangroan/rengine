extern crate rengine;

use rengine::comp::MeshCommandBuffer;
use rengine::specs::{Builder, RunNow, SystemData, World};
use rengine::voxel::{
    ChunkControl, ChunkMapping, ChunkUpkeepSystem, NoOpVoxelMeshGen, VoxelArrayChunk, VoxelChunk,
};

type IntVoxel = u16;
type IntVoxelChunk = VoxelArrayChunk<IntVoxel>;
type IntChunkCtrl = ChunkControl<u16, IntVoxelChunk>;
type IntUpkeepSystem = ChunkUpkeepSystem<IntVoxel, IntVoxelChunk, NoOpVoxelMeshGen>;

/// Ensure update queue is drained on maintain
#[test]
fn test_lazy_update() {
    let mut chunk_map = ChunkMapping::new();
    let mut ctrl: IntChunkCtrl = Default::default();
    ctrl.lazy_update([0, 0, 0], 1);
    ctrl.lazy_update([1, 0, 0], 2);
    ctrl.lazy_update([2, 0, 0], 3);
    assert_eq!(3, ctrl.cmd_len());

    // ECS integration
    let mut world = World::new();
    world.register::<IntVoxelChunk>();
    let entity = world
        .create_entity()
        .with(IntVoxelChunk::new([0, 0, 0]))
        .build();
    chunk_map.add_chunk(
        entity,
        world
            .read_storage::<IntVoxelChunk>()
            .get(entity)
            .unwrap()
            .index()
            .clone(),
    );
    world.add_resource(ctrl);
    world.add_resource(chunk_map);
    world.add_resource(MeshCommandBuffer::new());

    // Systems
    let mut upkeep_system: IntUpkeepSystem = IntUpkeepSystem::new(NoOpVoxelMeshGen);

    // Maintain
    upkeep_system.run_now(&mut world.res);
    assert_eq!(0, world.read_resource::<IntChunkCtrl>().cmd_len());

    {
        let chunks = world.read_storage::<IntVoxelChunk>();
        assert_eq!(Some(&1), chunks.get(entity).and_then(|c| c.get([0, 0, 0])));
        assert_eq!(Some(&2), chunks.get(entity).and_then(|c| c.get([1, 0, 0])));
        assert_eq!(Some(&3), chunks.get(entity).and_then(|c| c.get([2, 0, 0])));
    }
}
