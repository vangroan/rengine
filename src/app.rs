use crate::colors;
use crate::comp::Z_AXIS;
use crate::comp::{Mesh, MeshBuilder, Transform};
use crate::gfx_types::*;
use crate::graphics::{ChannelPair, GraphicContext};
use crate::res::DeltaTime;
use crate::sys::DrawSystem;
use gfx::traits::FactoryExt;
use gfx::Device;
use glutin::{Api, ContextBuilder, EventsLoop, GlProfile, GlRequest, WindowBuilder};
use specs::{Dispatcher, DispatcherBuilder, RunNow, World};
use std::error::Error;
use std::fmt;
use std::time::Instant;

/// The main application wrapper
#[allow(dead_code)]
pub struct App<'comp, 'thread> {
    graphics: GraphicContext,
    world: World,
    dispatcher: Dispatcher<'comp, 'thread>,
    bkg_color: colors::Color,
}

impl<'a, 'b> App<'a, 'b> {
    /// Starts the application loop
    ///
    /// Consumes the app
    pub fn run(self) -> AppResult {
        use glutin::Event::*;

        let App {
            mut graphics,
            mut world,
            mut dispatcher,
            bkg_color,
            ..
        } = self;

        // Pipeline State Object
        let pso: PipelineStateObject = graphics
            .factory
            .create_pipeline_simple(
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/shaders/basic_150.glslv"
                )),
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/shaders/basic_150.glslf"
                )),
                pipe::new(),
            )
            .unwrap();

        // Test Quad
        use specs::Builder;
        world.add_resource(pso);
        world.register::<Mesh>();
        world.register::<Transform>();
        let _entity = world
            .create_entity()
            .with(
                MeshBuilder::new()
                    .quad(
                        [0., 0., 0.],
                        [1., 1.],
                        [colors::RED, colors::GREEN, colors::BLUE, colors::MAGENTA],
                    )
                    .build(&mut graphics),
            )
            .with(
                Transform::default()
                    .with_anchor([0.5, 0.5, 0.0])
                    .with_position([0.0, 0.0, 0.])
                    .with_scale([0.5, 1.0, 1.0])
                    .with_rotation(10. * (::std::f32::consts::PI / 180.), Z_AXIS),
            )
            .build();

        // Encoder
        let mut channel = ChannelPair::new();
        if let Err(_) = channel.send_block(graphics.create_encoder()) {
            return Err(AppError::EncoderSend);
        }

        // Renderer
        // TODO: Consider having a `Renderer` trait since it's being treated differently than other systems
        let mut renderer = DrawSystem::new(channel.clone(), graphics.render_target.clone());

        // Loop control
        let mut running = true;
        let mut last_time = Instant::now();

        while running {
            // Time elapsed since last iteration
            let new_time = Instant::now();
            let delta_time = DeltaTime(new_time.duration_since(last_time));
            last_time = new_time;

            // Prepare world with frame scoped resources
            world.add_resource(delta_time);

            // Drain user input events
            graphics.events_loop.poll_events(|event| match event {
                WindowEvent {
                    event: glutin::WindowEvent::CloseRequested,
                    ..
                } => running = false,
                _ => (),
            });

            // Pre-render
            match channel.recv_block() {
                Ok(mut encoder) => {
                    encoder.clear(&graphics.render_target, bkg_color);

                    // Send encoder back
                    channel.send_block(encoder)?;
                }
                Err(_) => return Err(AppError::EncoderRecv),
            }

            // Run systems
            dispatcher.dispatch(&world.res);

            // Render Components
            renderer.run_now(&world.res);

            // Commit Render
            match channel.recv_block() {
                Ok(mut encoder) => {
                    // encoder.draw(&slice, &pso, &data);
                    encoder.flush(&mut graphics.device);
                    graphics.window.swap_buffers().unwrap();

                    // Send encoder back
                    channel.send_block(encoder)?;
                }
                Err(_) => return Err(AppError::EncoderRecv),
            }

            // Deallocate
            graphics.device.cleanup();
            world.maintain();
        }

        Ok(())
    }
}

pub type AppResult = Result<(), AppError>;

#[derive(Debug)]
#[must_use]
pub enum AppError {
    /// Graphics encoder was not in the channel when render occurred
    EncoderRecv,

    /// Graphics encoder could not be sent over the channel, possibly because it has been disconnected
    EncoderSend,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AppError::*;

        write!(
            f,
            "Application Error, {}",
            match self {
                EncoderRecv => "Encoder Receive",
                EncoderSend => "Encoder Send",
            }
        )
    }
}

impl Error for AppError {
    fn description(&self) -> &str {
        use AppError::*;

        match self {
            EncoderRecv => "Graphics encoder was not received from channel",
            EncoderSend => "Graphics encoder could not be sent to the channel",
        }
    }
}

impl<R, C> From<crossbeam::channel::SendError<gfx::Encoder<R, C>>> for AppError
where
    R: gfx::Resources,
    C: gfx::CommandBuffer<R>,
{
    fn from(_send_error: crossbeam::channel::SendError<gfx::Encoder<R, C>>) -> Self {
        AppError::EncoderSend
    }
}

/// Builder for application
///
/// Usage:
///
/// ```ignore
/// extern crate rengine;
///
/// let app = rengine::AppBuilder::new()
///     .size(640, 480)
///     .title("Example App")
///     .build()
///     .unwrap();
/// ```
pub struct AppBuilder {
    size: [u32; 2],
    title: &'static str,
    bkg_color: colors::Color,
}

impl AppBuilder {
    pub fn new() -> Self {
        AppBuilder {
            size: [640, 480],
            title: "rengine",
            bkg_color: colors::BLACK,
        }
    }

    /// The initial size of the window
    #[inline]
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = [width, height];
        self
    }

    /// The title displayed at the top of the window
    #[inline]
    pub fn title(mut self, title: &'static str) -> Self {
        self.title = title;
        self
    }

    /// The default color used as the background of the window
    #[inline]
    pub fn background_color(mut self, color: colors::Color) -> Self {
        self.bkg_color = color;
        self
    }

    /// Consumes the builder and creates the application
    pub fn build<'a, 'b>(self) -> Result<App<'a, 'b>, Box<dyn Error>> {
        // Event Loop
        let events_loop = EventsLoop::new();

        // Window
        let window_builder = WindowBuilder::new()
            .with_title(self.title)
            .with_dimensions((self.size[0], self.size[1]).into());

        // OpenGL Context
        let context_builder = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 2)))
            .with_gl_profile(GlProfile::Core) // modern OpenGL only
            .with_vsync(true);

        // Init
        let (window, device, factory, render_target, depth_stencil) =
            gfx_glutin::init::<ColorFormat, DepthFormat>(
                window_builder,
                context_builder,
                &events_loop,
            )?;

        // Graphics Context
        let graphics = GraphicContext {
            events_loop,
            window,
            device,
            factory,
            render_target,
            depth_stencil,
        };

        // World
        let world = World::new();

        // Dispatcher
        let dispatcher = DispatcherBuilder::new().build();

        Ok(App {
            graphics,
            world,
            dispatcher,
            bkg_color: self.bkg_color,
        })
    }
}
