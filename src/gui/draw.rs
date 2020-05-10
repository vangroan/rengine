use crate::draw2d::Canvas;
use crate::gfx_types::{DepthTarget, PipelineBundle, RenderTarget};
use crate::graphics::GraphicContext;
use crate::gui::GuiDrawable;
use crate::render::ChannelPair;
use gfx_device::{CommandBuffer, Resources};
use specs::{Join, ReadExpect, ReadStorage, System};

pub struct DrawGuiSystem {
    channel: ChannelPair<Resources, CommandBuffer>,
    canvas: Canvas,
    render_target: RenderTarget<gfx_device::Resources>,
    depth_target: DepthTarget<gfx_device::Resources>,
}

impl DrawGuiSystem {
    pub fn new(
        channel: ChannelPair<Resources, CommandBuffer>,
        canvas: Canvas,
        render_target: RenderTarget<gfx_device::Resources>,
        depth_target: DepthTarget<gfx_device::Resources>,
    ) -> Self {
        DrawGuiSystem {
            channel,
            canvas,
            render_target,
            depth_target,
        }
    }
}

impl<'a> System<'a> for DrawGuiSystem {
    type SystemData = (
        // ReadExpect<'a, PipelineBundle<pipe::Meta>>,
        ReadStorage<'a, GuiDrawable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (drawables,) = data;

        match self.channel.recv_block() {
            Ok(_encoder) => {
                for (drawable,) in (&drawables,).join() {
                    match drawable {
                        GuiDrawable::Text(_txt) => draw_txt(),
                        GuiDrawable::Rectangle(_rect) => { /* Draw to canvas */ }
                    }
                }

                // Draw to screen

                self.channel
                    .send_block(_encoder)
                    .expect("GUI render failed sending encoder back to main loop");
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}

fn draw_txt() {}
