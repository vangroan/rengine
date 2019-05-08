extern crate rengine;

use rengine::specs::{Builder, World};
use rengine::voxel::{ChunkControl, VoxelArrayChunk, VoxelChunk};

type IntVoxelChunk = VoxelArrayChunk<u16>;

/// Ensure update queue is drained on maintain
#[test]
fn test_lazy_update() {
    let mut ctrl: ChunkControl<u16, IntVoxelChunk> = Default::default();
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
    ctrl.add_chunk(
        entity,
        world.read_storage::<IntVoxelChunk>().get(entity).unwrap(),
    );

    // Maintain
    ctrl.maintain(&mut world.write_storage::<IntVoxelChunk>());
    assert_eq!(0, ctrl.cmd_len());

    {
        let chunks = world.read_storage::<IntVoxelChunk>();
        assert_eq!(Some(&1), chunks.get(entity).and_then(|c| c.get([0, 0, 0])));
        assert_eq!(Some(&2), chunks.get(entity).and_then(|c| c.get([1, 0, 0])));
        assert_eq!(Some(&3), chunks.get(entity).and_then(|c| c.get([2, 0, 0])));
    }
}
