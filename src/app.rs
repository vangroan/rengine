use crate::angle::Deg;
use crate::colors;
use crate::comp::{Camera, GlTexture, Mesh, MeshBuilder, Transform};
use crate::comp::{X_AXIS, Y_AXIS};
use crate::gfx_types::*;
use crate::graphics::{ChannelPair, GraphicContext};
use crate::res::{ActiveCamera, DeltaTime, DeviceDimensions, ViewPort};
use crate::scene::{SceneBuilder, SceneFactories, SceneStack};
use crate::sys::{CameraResizeSystem, DrawSystem};
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
    events_loop: EventsLoop,
    graphics: GraphicContext,
    world: World,
    dispatcher: Dispatcher<'comp, 'thread>,
    bkg_color: colors::Color,
    initial_scene_key: Option<&'static str>,
    scene_factories: SceneFactories,
}

impl<'a, 'b> App<'a, 'b> {
    pub fn init_scene(&mut self, key: &'static str) {
        self.initial_scene_key = Some(key);
    }

    pub fn register_scene<F>(&mut self, key: &'static str, factory: F)
    where
        F: 'static + Fn(SceneBuilder) -> SceneBuilder,
    {
        self.scene_factories.add(key, factory);
    }

    /// Starts the application loop
    ///
    /// Consumes the app
    pub fn run(self) -> AppResult {
        use glutin::Event::*;

        let App {
            mut events_loop,
            mut graphics,
            mut world,
            mut dispatcher,
            initial_scene_key,
            scene_factories,
            bkg_color,
            ..
        } = self;

        // Engine Components
        world.register::<Mesh>();
        world.register::<Transform>();
        world.register::<Camera>();
        world.register::<GlTexture>();

        // Assets
        // TODO: Place in world and allow for loading textures from game without needing factory (operation buffer?)
        let mut textures = GraphicContext::create_texture_cache();

        // Initial ViewPort Size
        let device_dimensions = match DeviceDimensions::from_window(&graphics.window) {
            Some(dim) => dim,
            None => return Err(AppError::WindowSize),
        };

        // Implementation of Into<(u32, u2)> performs proper rounding
        let (logical_w, logical_h): (u32, u32) = device_dimensions.logical_size.into();
        let (physical_w, physical_h): (u32, u32) = device_dimensions.physical_size.into();

        world.add_resource(ViewPort::new((physical_w as u16, physical_h as u16)));
        world.add_resource(device_dimensions);

        // Default Camera
        let camera_entity = world
            .create_entity()
            .with(Transform::new().with_position([0., 0., -2.]))
            .with({
                let mut cam = Camera::with_device_size((logical_w as u16, logical_h as u16));
                cam.set_target([0., 0., 2.]);

                cam
            })
            .build();
        world.add_resource(ActiveCamera::new(camera_entity));

        // Update Camera on Resize
        // TODO: message passing to notify systems of events
        let mut camera_resize_system = CameraResizeSystem::new();

        // Shader program
        let shader_program = graphics
            .factory
            .link_program(
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/shaders/basic_150.glslv"
                )),
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/shaders/basic_150.glslf"
                )),
            )
            .unwrap();

        // Pipeline State Object
        let pso = graphics
            .factory
            .create_pipeline_from_program(
                &shader_program,
                gfx::Primitive::TriangleList,
                gfx::state::Rasterizer::new_fill().with_cull_back(),
                pipe::new(),
            )
            .unwrap();

        // Bundle program and pipeline state object together to avoid
        // lifetime issues with world resources borrowing each other.
        world.add_resource(PipelineBundle::new(pso, shader_program));

        // Test Quad
        use specs::Builder;
        let tex = GlTexture::from_bundle(
            textures.load_texture(&mut graphics.factory, "examples/block.png"),
        );
        let tex_rects = {
            let tex_rect = tex.source_rect();
            let back_rect = tex_rect.sub_rect([0, 0], [16, 16]);
            let front_rect = tex_rect.sub_rect([16, 0], [16, 16]);
            let left_rect = tex_rect.sub_rect([32, 0], [16, 16]);
            let right_rect = tex_rect.sub_rect([0, 16], [16, 16]);
            let bottom_rect = tex_rect.sub_rect([16, 16], [16, 16]);
            let top_rect = tex_rect.sub_rect([32, 16], [16, 16]);
            [
                back_rect,
                front_rect,
                left_rect,
                right_rect,
                bottom_rect,
                top_rect,
            ]
        };
        let _entity = world
            .create_entity()
            .with(
                MeshBuilder::new()
                    // .quad(
                    //     [0., 0., 0.],
                    //     [1., 1.],
                    //     // [colors::RED, colors::GREEN, colors::BLUE, colors::MAGENTA],
                    //     [colors::WHITE, colors::WHITE, colors::WHITE, colors::WHITE],
                    // )
                    .pseudocube([0., 0., 0.], [1., 1., 1.], tex_rects)
                    .build(&mut graphics),
            )
            .with(
                Transform::default()
                    .with_anchor([0.5, 0.5, 0.5])
                    .with_position([0.25, 0.25, 0.])
                    .with_scale([0.5, 0.5, 0.5])
                    .with_rotate_world(Deg(45.), Y_AXIS)
                    .with_rotate_world(Deg(30.), X_AXIS),
            )
            .with(tex)
            .build();

        // Encoder
        let mut channel = ChannelPair::new();
        if let Err(_) = channel.send_block(graphics.create_encoder()) {
            return Err(AppError::EncoderSend);
        }

        // Renderer
        // TODO: Consider having a `Renderer` trait since it's being treated differently than other systems
        let mut renderer = DrawSystem::new(channel.clone(), graphics.render_target.clone());

        // Scenes
        let mut scene_stack = SceneStack::new(scene_factories);

        match initial_scene_key {
            Some(key) => {
                scene_stack.push(key);
            }
            None => return Err(AppError::NoInitialScene),
        }

        // Loop control
        let mut running = true;
        let mut last_time = Instant::now();

        while running {
            // Time elapsed since last iteration
            let new_time = Instant::now();
            let delta_time = DeltaTime(new_time.duration_since(last_time));
            last_time = new_time;

            // Prepare requested scene
            scene_stack.maintain();

            // Prepare world with frame scoped resources
            world.add_resource(delta_time);

            // Drain user input events
            events_loop.poll_events(|event| match event {
                WindowEvent {
                    event: glutin::WindowEvent::CloseRequested,
                    ..
                } => running = false,
                WindowEvent {
                    event: glutin::WindowEvent::Resized(logical_size),
                    ..
                } => {
                    // Coordinates use physical size
                    let dpi_factor = graphics.window.get_hidpi_factor();
                    let physical_size = logical_size.to_physical(dpi_factor);

                    // Required by some platforms
                    graphics.window.resize(physical_size);

                    // Update dimensions of frame buffer targets
                    graphics.update_views();

                    // Ensure no dangling shared references
                    renderer.render_target = graphics.render_target.clone();

                    // Update view port/scissor rectangle for rendering systems
                    let (win_w, win_h): (u32, u32) = physical_size.into();
                    let vp = ViewPort::new((win_w as u16, win_h as u16));
                    world.add_resource(vp);

                    // Update cameras
                    world.add_resource(DeviceDimensions::new(dpi_factor, logical_size));
                    camera_resize_system.run_now(&world.res);
                }
                _ => (),
            });

            // TODO: Remove
            {
                use specs::Join;
                let mut trans = world.write_storage::<Transform>();
                for (ref mut tran,) in (&mut trans,).join() {
                    tran.rotate(Deg(0.5), Y_AXIS);
                }
            }

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

    /// Failed to retrieve Window Size
    WindowSize,

    NoInitialScene,
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
                WindowSize => "Window Size",
                NoInitialScene => "No initial Scene",
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
            WindowSize => "Failed to retrieve window size",
            NoInitialScene => "No initial scene configured",
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
            events_loop,
            graphics,
            world,
            dispatcher,
            bkg_color: self.bkg_color,
            initial_scene_key: None,
            scene_factories: SceneFactories::new(),
        })
    }
}
