use crate::camera::{ActiveCamera, CameraProjection, CameraResizeSystem, CameraView};
use crate::colors;
use crate::comp::{GlTexture, Mesh, Transform};
use crate::gfx_types::*;
use crate::graphics::GraphicContext;
use crate::render::{ChannelPair, GizmoDrawSystem, GizmoPipelineBundle};
use crate::res::{DeltaTime, DeviceDimensions, ViewPort};
use crate::scene::{Scene, SceneError, SceneStack};
use crate::sys::DrawSystem;
use gfx::traits::FactoryExt;
use gfx::Device;
use glutin::{Api, ContextBuilder, EventsLoop, GlProfile, GlRequest, WindowBuilder};
use specs::{Builder, Dispatcher, DispatcherBuilder, RunNow, World};
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
    initial_scene: Option<Box<Scene>>,
}

impl<'a, 'b> App<'a, 'b> {
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
            initial_scene,
            bkg_color,
            ..
        } = self;

        // Engine Components
        world.register::<Mesh>();
        world.register::<Transform>();
        world.register::<CameraView>();
        world.register::<CameraProjection>();
        world.register::<GlTexture>();

        // Assets
        // TODO: Place in world and allow for loading textures from game without needing factory (operation buffer?)
        let textures = GraphicContext::create_texture_cache();
        world.add_resource(textures);

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
            .with(CameraProjection::with_device_size((
                logical_w as u16,
                logical_h as u16,
            )))
            .with(CameraView::new())
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

        // Gizmo Wireframe PSO
        {
            let wireframe_shader = graphics
                .factory
                .link_program(
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/src/shaders/gizmo_150.glslv"
                    )),
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/src/shaders/gizmo_150.glslf"
                    )),
                )
                .unwrap();

            let wireframe_pso = graphics
                .factory
                .create_pipeline_from_program(
                    &wireframe_shader,
                    gfx::Primitive::TriangleList,
                    gfx::state::Rasterizer::new_fill().with_cull_back(),
                    pipe::new(),
                )
                .unwrap();

            world.add_resource(GizmoPipelineBundle::new(wireframe_pso, wireframe_shader));
        }

        // Encoder
        let mut channel = ChannelPair::new();
        if let Err(_) = channel.send_block(graphics.create_encoder()) {
            return Err(AppError::EncoderSend);
        }

        // Renderer
        // TODO: Consider having a `Renderer` trait since it's being treated differently than other systems
        let mut renderer = DrawSystem::new(channel.clone(), graphics.render_target.clone());
        let mut gizmo_renderer =
            GizmoDrawSystem::new(channel.clone(), graphics.render_target.clone());

        // Scenes
        let mut scene_stack = SceneStack::new();

        match initial_scene {
            Some(scene_box) => {
                scene_stack.push_box(scene_box);
            }
            None => return Err(AppError::NoInitialScene),
        }

        // Loop control
        let mut running = true;
        let mut last_time = Instant::now();

        'main: while running {
            // Time elapsed since last iteration
            let new_time = Instant::now();
            let delta_time = DeltaTime(new_time.duration_since(last_time));
            last_time = new_time;

            // Prepare requested scene
            scene_stack.maintain(&mut world, &mut graphics)?;

            // Prepare world with frame scoped resources
            world.add_resource(delta_time);

            // Drain user input events
            events_loop.poll_events(|event| {
                // Global event handling
                match event {
                    WindowEvent {
                        event: glutin::WindowEvent::CloseRequested,
                        ..
                    } => {
                        println!("Shutting down");

                        running = false;

                        // Allow scenes to cleanup resources
                        if let Err(err) = scene_stack.clear(&mut world, &mut graphics) {
                            eprintln!("{:?}", err);
                        }
                    }
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
                }

                // Scene event handling
                scene_stack.dispatch_event(&mut world, &mut graphics, &event);
            });

            // Scene Update
            scene_stack.dispatch_update(&mut world, &mut graphics);

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

    /// App was setup with no initial scene
    NoInitialScene,
    SceneTransitionFail(SceneError),
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
                SceneTransitionFail(_) => "Scene Transition Fail",
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
            SceneTransitionFail(_) => "Failure to transition scene during maintenance phase",
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use AppError::*;

        match self {
            SceneTransitionFail(err) => Some(err),
            _ => None,
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

impl From<SceneError> for AppError {
    fn from(scene_error: SceneError) -> Self {
        AppError::SceneTransitionFail(scene_error)
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
    initial_scene: Option<Box<dyn Scene>>,
}

impl AppBuilder {
    pub fn new() -> Self {
        AppBuilder {
            size: [640, 480],
            title: "rengine",
            bkg_color: colors::BLACK,
            initial_scene: None,
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

    #[inline]
    pub fn init_scene<S>(mut self, scene: S) -> Self
    where
        S: 'static + Scene,
    {
        self.initial_scene = Some(Box::new(scene));
        self
    }

    /// Consumes the builder and creates the application
    pub fn build<'a, 'b>(mut self) -> Result<App<'a, 'b>, Box<dyn Error>> {
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

        // Initial Scene
        let initial_scene = self.initial_scene.take();

        Ok(App {
            events_loop,
            graphics,
            world,
            dispatcher,
            bkg_color: self.bkg_color,
            initial_scene,
        })
    }
}
