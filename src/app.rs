use crate::colors;
use crate::gfx_types::*;
use crate::graphics::GraphicContext;
use crate::res::DeltaTime;
use gfx::traits::FactoryExt;
use gfx::Device;
use glutin::{Api, ContextBuilder, EventsLoop, GlProfile, GlRequest, WindowBuilder};
use specs::{Dispatcher, DispatcherBuilder, World};
use std::error::Error;
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
    pub fn run(self) {
        use glutin::Event::*;

        let App {
            mut graphics,
            mut world,
            mut dispatcher,
            bkg_color,
            ..
        } = self;

        // Pipeline Object
        let pso = graphics
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
        use crate::comp::Quad;
        use specs::Builder;
        world.register::<Quad>();
        let entity = world
            .create_entity()
            .with(Quad::new([0., 0., 0.], [1.0, 1.0]))
            .build();
        let (vertices, indices) = world
            .read_storage::<Quad>()
            .get(entity)
            .unwrap()
            .create_vertices_indices();

        let (vertex_buffer, slice) = graphics
            .factory
            .create_vertex_buffer_with_slice(&vertices, &*indices);

        let data = pipe::Data {
            vbuf: vertex_buffer,
            out: graphics.render_target,
        };

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

            // Run systems
            dispatcher.dispatch(&world.res);

            // Render
            graphics.encoder.clear(&data.out, bkg_color);
            graphics.encoder.draw(&slice, &pso, &data);
            graphics.encoder.flush(&mut graphics.device);
            graphics.window.swap_buffers().unwrap();

            // Deallocate
            graphics.device.cleanup();
            world.maintain();
        }
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
        let (window, device, mut factory, render_target, depth_stencil) =
            gfx_glutin::init::<ColorFormat, DepthFormat>(
                window_builder,
                context_builder,
                &events_loop,
            )?;

        // Encoder
        let encoder: gfx::Encoder<gfx_device::Resources, gfx_device::CommandBuffer> =
            factory.create_command_buffer().into();

        // Graphics Context
        let graphics = GraphicContext {
            events_loop,
            encoder,
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
