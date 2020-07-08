use crate::camera::{
    ActiveCamera, CameraProjection, CameraResizeSystem, CameraView, DollyCamera, FocusTarget,
    GridCamera, OrbitalCamera, SlideCamera,
};
use crate::colors;
use crate::comp::{GlTexture, Mesh, MeshCommandBuffer, MeshUpkeepSystem, Tag, Transform};
use crate::draw2d::Canvas;
use crate::errors::*;
use crate::gfx_types::*;
use crate::graphics::GraphicContext;
use crate::gui::{self, text, widgets, DrawGuiSystem, GuiGraph};
use crate::metrics::MetricHub;
use crate::modding::Mods;
use crate::render::{self, ChannelPair, Gizmo, Lights, Material, PointLight};
use crate::res::{DeltaTime, DeviceDimensions, ViewPort};
use crate::scene::{Scene, SceneStack};
use crate::sys::DrawSystem;
use crate::util;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx_glyph::{ab_glyph::FontArc, GlyphBrushBuilder};
use glutin::{Api, ContextBuilder, EventsLoop, GlProfile, GlRequest, WindowBuilder};
use log::{error, trace};
use specs::prelude::*;

use std::path::Path;
use std::time::Instant;

const DEFAULT_FONT_DATA: &[u8] = include_bytes!("../resources/fonts/DejaVuSans.ttf");

/// The main application wrapper
#[allow(dead_code)]
pub struct App<'comp, 'thread> {
    events_loop: EventsLoop,
    graphics: GraphicContext,
    world: World,
    dispatcher: Dispatcher<'comp, 'thread>,
    bkg_color: colors::Color,
    initial_scene: Option<Box<dyn Scene>>,
    mods: Option<(&'static str, &'static str)>,
}

impl<'a, 'b> App<'a, 'b> {
    /// The global world associated with the appliction.
    ///
    /// Used for registering application level component
    /// types and resources, intended to be used by the
    /// whole game.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// app.world().add_resource(Myresource::new());
    /// app.world().register::<MyComponent>();
    /// ```
    #[inline]
    pub fn world(&mut self) -> &mut World {
        &mut self.world
    }

    /// Starts the application loop
    ///
    /// Consumes the app
    pub fn run(self) -> Result<()> {
        use glutin::Event::*;

        let App {
            mut events_loop,
            mut graphics,
            mut world,
            mut dispatcher,
            initial_scene,
            bkg_color,
            mods,
            ..
        } = self;

        // Engine Components
        world.register::<Mesh>();
        world.register::<Transform>();
        world.register::<Material>();
        world.register::<PointLight>();
        world.register::<Gizmo>();
        world.register::<CameraView>();
        world.register::<CameraProjection>();
        world.register::<FocusTarget>();
        world.register::<OrbitalCamera>();
        world.register::<GridCamera>();
        world.register::<DollyCamera>();
        world.register::<SlideCamera>();
        world.register::<GlTexture>();
        world.register::<Tag>();
        world.register::<util::FpsCounter>();

        // GUI Components
        {
            world.add_resource(gui::HoveredWidget::default());
            world.add_resource(gui::PressedWidget::default());
            world.add_resource(gui::WidgetEvents::new());
            world.register::<gui::GuiMesh>();
            world.register::<gui::BoundsRect>();
            world.register::<gui::Placement>();
            world.register::<gui::Pack>();
            world.register::<gui::GlobalPosition>();
            world.register::<gui::Clickable>();
            world.register::<gui::ZDepth>();
            world.register::<gui::text::TextBatch>();
            world.register::<widgets::Button>();
            world.register::<widgets::Container>();
        }

        // Statistics Metrics
        world.add_resource(MetricHub::default());

        // Event Streams
        world.add_resource::<Vec<glutin::Event>>(Vec::new());

        // Lights
        world.add_resource(Lights::new(&mut graphics, render::MAX_NUM_LIGHTS));

        // GUI
        let root_entity = widgets::create_container(&mut world, gui::PackMode::Frame);
        let gui_graph = GuiGraph::with_root(root_entity);
        world.add_resource(gui::LayoutDirty::with_node_id(gui_graph.root_id())); // Initial layout pass
        world.add_resource(gui_graph);

        // Graphics Commands to allow allocating resources
        // from systems to draw thread.
        world.add_resource(MeshCommandBuffer::new());
        let mesh_upkeep = MeshUpkeepSystem;

        // Assets
        // TODO: Place in world and allow for loading textures from game without needing factory (operation buffer?)
        let textures = GraphicContext::create_texture_cache();
        world.add_resource(textures);

        // Initial ViewPort Size
        let device_dimensions = match DeviceDimensions::from_window(&graphics.window) {
            Some(dim) => dim,
            None => return Err(ErrorKind::WindowSize.into()),
        };

        // Implementation of Into<(u32, u2)> performs proper rounding
        let (logical_w, logical_h): (u32, u32) = device_dimensions.logical_size.into();
        let (physical_w, physical_h): (u32, u32) = device_dimensions.physical_size.into();

        world.add_resource(ViewPort::new((physical_w as u16, physical_h as u16)));
        world.add_resource(device_dimensions);

        // Default Camera
        let camera_entity = world
            .create_entity()
            .with(Transform::new().with_position([0., 0., 2.]))
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

        // Basic render PSO
        {
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
        }

        // Gloss Material PSO
        {
            // Shader program
            let shader_program = graphics
                .factory
                .link_program(
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/src/shaders/gloss_150.glslv"
                    )),
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/src/shaders/gloss_150.glslf"
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
                    gloss_pipe::new(),
                )
                .expect("Failed to link gloss material shader");

            // Bundle program and pipeline state object together to avoid
            // lifetime issues with world resources borrowing each other.
            world.add_resource(PipelineBundle::new(pso, shader_program));
        }

        // Gizmo Wireframe PSO
        {
            let gizmo_shader = graphics
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

            let mut fillmode = gfx::state::Rasterizer::new_fill();
            fillmode.method = gfx::state::RasterMethod::Line(1); // Render lines
            let gizmo_pso = graphics
                .factory
                .create_pipeline_from_program(
                    &gizmo_shader,
                    gfx::Primitive::TriangleList,
                    fillmode,
                    gizmo_pipe::new(),
                )
                .expect("Failed to link wireframe material shader");

            world.add_resource(PipelineBundle::new(gizmo_pso, gizmo_shader));
        }

        // GUI PSO
        {
            let gui_shader = graphics
                .factory
                .link_program(
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/src/shaders/gui_150.glslv"
                    )),
                    include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/src/shaders/gui_150.glslf"
                    )),
                )
                .unwrap();

            // GUI PSO
            let pso = graphics
                .factory
                .create_pipeline_from_program(
                    &gui_shader,
                    gfx::Primitive::TriangleList,
                    // TODO: Currently we're drawing quads backwards
                    gfx::state::Rasterizer::new_fill().with_cull_back(),
                    gui_pipe::new(),
                )
                .unwrap();

            // Bundle program and pipeline state object together to avoid
            // lifetime issues with world resources borrowing each other.
            world.add_resource(PipelineBundle::new(pso, gui_shader));
        }

        // Encoder
        let mut channel = ChannelPair::new();
        channel.send_block(graphics.create_encoder())?;

        // Renderer
        // TODO: Consider having a `Renderer` trait since it's being treated differently than other systems
        let mut renderer = DrawSystem::new(
            channel.clone(),
            graphics.render_target.clone(),
            graphics.depth_stencil.clone(),
        );

        // Text Rendering
        let default_font = FontArc::try_from_slice(DEFAULT_FONT_DATA).unwrap();
        let mut text_renderer = text::DrawTextSystem::new(
            channel.clone(),
            graphics.render_target.clone(),
            graphics.depth_stencil.clone(),
            GlyphBrushBuilder::using_font(default_font)
                .depth_test(gfx::preset::depth::LESS_EQUAL_WRITE)
                .build(graphics.factory.clone()),
        );

        // Gui Rendering
        let mut gui_renderer = DrawGuiSystem::new(
            channel.clone(),
            Canvas::new(&mut graphics, physical_w as u16, physical_h as u16).unwrap(),
            graphics.render_target.clone(),
            graphics.depth_stencil.clone(),
        );

        // Modding
        if let Some((lib_name, mod_path)) = mods {
            let path = Path::new(mod_path);
            trace!(
                "Initialising Modding. Library name: {}, Path: {}",
                lib_name,
                path.to_str().unwrap()
            );

            world.add_resource(Mods::new(lib_name, path));
        }

        // Scenes
        let mut scene_stack = SceneStack::new();

        match initial_scene {
            Some(scene_box) => {
                scene_stack.push_box(scene_box);
            }
            None => return Err(ErrorKind::NoInitialScene.into()),
        }

        // Loop control
        let mut running = true;
        let mut last_time = Instant::now();

        // Buffer to copy events into, to avoid having to borrow
        // event stream from world.
        let mut events: Vec<glutin::Event> = Vec::new();

        while running {
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
                events.push(event.clone());

                // Global event handling
                match event {
                    WindowEvent {
                        event: glutin::WindowEvent::CloseRequested,
                        ..
                    } => {
                        trace!("Shutting down");

                        running = false;

                        // Allow scenes to cleanup resources
                        if let Err(err) = scene_stack.clear(&mut world, &mut graphics) {
                            error!("{:?}", err);
                        }
                    }
                    WindowEvent {
                        event: glutin::WindowEvent::Resized(logical_size),
                        ..
                    } => {
                        // Coordinates use physical size
                        let dpi_factor = graphics.window.window().get_hidpi_factor();
                        let physical_size = logical_size.to_physical(dpi_factor);
                        // println!("dpi_factor={} {:?} {:?}", dpi_factor, physical_size, logical_size);

                        // Required by some platforms
                        graphics.window.resize(physical_size);

                        // Update dimensions of frame buffer targets
                        graphics.update_views();

                        // Ensure no dangling shared references
                        renderer.render_target = graphics.render_target.clone();
                        renderer.depth_target = graphics.depth_stencil.clone();
                        text_renderer.render_target = graphics.render_target.clone();
                        text_renderer.depth_target = graphics.depth_stencil.clone();
                        gui_renderer.render_target = graphics.render_target.clone();
                        gui_renderer.depth_target = graphics.depth_stencil.clone();

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

            world.exec(|(mut event_stream,): (specs::Write<Vec<glutin::Event>>,)| {
                event_stream.extend(events.drain(..));
            });

            // Scene Update
            scene_stack.dispatch_update(&mut world, &mut graphics);

            // Pre-render
            {
                let mut encoder = channel.recv_block()?;
                encoder.clear(&graphics.render_target, bkg_color);
                encoder.clear_depth(&graphics.depth_stencil, 1.0);

                // Send encoder back
                channel.send_block(encoder)?;
            }

            // Run systems
            dispatcher.dispatch(&world.res);

            // Allocate Graphic Resources
            mesh_upkeep.maintain(&mut graphics, world.system_data());

            // Render Components
            renderer.run_now(&world.res);

            // Render Gui
            gui_renderer.run_now(&world.res);

            // Render Text
            text_renderer.run_now(&world.res);

            // Commit Render
            {
                let mut encoder = channel.recv_block()?;
                encoder.flush(&mut graphics.device);
                graphics.window.swap_buffers().unwrap();

                // Send encoder back
                channel.send_block(encoder)?;
            }

            // Deallocate
            graphics.device.cleanup();
            world.maintain();

            // Flush event stream
            world.exec(|(mut event_stream,): (specs::Write<Vec<glutin::Event>>,)| {
                event_stream.clear();
            });

            // Cooperatively give up CPU time
            // ::std::thread::yield_now();

            // TODO: Remove sleep; call update and render on separate timers
            ::std::thread::sleep(::std::time::Duration::from_millis(1));
        }

        Ok(())
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
    mods: Option<(&'static str, &'static str)>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        AppBuilder {
            size: [640, 480],
            title: "rengine",
            bkg_color: colors::BLACK,
            initial_scene: None,
            mods: None,
        }
    }
}

impl AppBuilder {
    pub fn new() -> Self {
        Default::default()
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

    #[inline]
    pub fn add_modding(mut self, lib_name: &'static str, mod_path: &'static str) -> Self {
        self.mods = Some((lib_name, mod_path));
        self
    }

    /// Consumes the builder and creates the application
    pub fn build<'a, 'b>(mut self) -> Result<App<'a, 'b>> {
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

        // Text Rendering
        let default_font = FontArc::try_from_slice(DEFAULT_FONT_DATA).unwrap();
        let glyph_brush = GlyphBrushBuilder::using_font(default_font).build(factory.clone());

        // Graphics Context
        let graphics = GraphicContext {
            window,
            device,
            factory,
            render_target,
            depth_stencil,
            glyph_brush,
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
            mods: self.mods.take(),
        })
    }
}
