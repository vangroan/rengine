use crate::gui::GuiDrawable;
use crate::render::ChannelPair;
use gfx_device::{CommandBuffer, Resources};
use specs::{Join, ReadStorage, System};
use std::error::Error;

pub struct DrawGuiSystem {
    channel: ChannelPair<Resources, CommandBuffer>,
}

impl DrawGuiSystem {
    pub fn new(channel: ChannelPair<Resources, CommandBuffer>) -> Self {
        DrawGuiSystem { channel }
    }
}

impl<'a> System<'a> for DrawGuiSystem {
    type SystemData = (ReadStorage<'a, GuiDrawable>,);

    fn run(&mut self, data: Self::SystemData) {
        let (drawables,) = data;

        match self.channel.recv_block() {
            Ok(_encoder) => {
                for (drawable,) in (&drawables,).join() {
                    match drawable {
                        GuiDrawable::Text(_txt) => draw_txt(),
                        GuiDrawable::Rectangle(_rect) => {}
                    }
                }

                self.channel
                    .send_block(_encoder)
                    .expect("GUI render failed sending encoder back to main loop");
            }
            Err(err) => eprintln!("{}, {}", err, err.description()),
        }
    }
}

fn draw_txt() {}
