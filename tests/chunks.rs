extern crate rengine;

use rengine::specs::{Builder, World, WriteStorage};
use rengine::voxel::{ChunkControl, VoxelArrayChunk};

type IntVoxelChunk = VoxelArrayChunk<u16>;

/// Ensure update queue is drained on maintain
#[test]
fn test_lazy_update() {
    let mut ctrl: ChunkControl<u16, IntVoxelChunk> = Default::default();
    ctrl.lazy_update([0, 0, 0], 1);
    ctrl.lazy_update([0, 0, 0], 1);
    ctrl.lazy_update([0, 0, 0], 1);
    assert_eq!(3, ctrl.cmd_len());

    // ECS integration
    let mut world = World::new();
    world.register::<IntVoxelChunk>();
    let entity = world
        .create_entity()
        .with(IntVoxelChunk::new([0, 0, 0]))
        .build();

    // Maintain
    ctrl.maintain(&mut world.write_storage::<IntVoxelChunk>());
    assert_eq!(0, ctrl.cmd_len());
}
