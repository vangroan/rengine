use crate::comp::Mesh;
use crate::graphics::ChannelPair;
use specs::{Join, ReadStorage, System};
use std::error::Error;

pub struct DrawSystem<R: gfx::Resources, C: gfx::CommandBuffer<R>> {
    channel: ChannelPair<R, C>,
}

impl<R, C> DrawSystem<R, C>
where
    R: gfx::Resources,
    C: gfx::CommandBuffer<R>,
{
    pub fn new(channel: ChannelPair<R, C>) -> Self {
        DrawSystem { channel }
    }
}

impl<'a, R, C> System<'a> for DrawSystem<R, C>
where
    R: gfx::Resources,
    C: gfx::CommandBuffer<R>,
{
    type SystemData = (ReadStorage<'a, Mesh>,);

    fn run(&mut self, (meshes,): Self::SystemData) {
        match self.channel.recv_block() {
            Ok(mut encoder) => {
                (&meshes,).join();

                if let Err(err) = self.channel.send_block(encoder) {
                    eprintln!("{}, {}", err, err.description());
                }
            }
            Err(err) => eprintln!("{}, {}", err, err.description()),
        }
    }
}
