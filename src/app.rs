use std::error::Error;
use winit::{EventsLoop, Window, WindowBuilder};

/// The main application wrapper
#[allow(dead_code)]
#[derive(Debug)]
pub struct App {
    events_loop: EventsLoop,
    window: Window,
}

impl App {
    fn new(events_loop: EventsLoop, window: Window) -> Self {
        App {
            events_loop,
            window,
        }
    }

    /// Starts the application loop
    pub fn run(&mut self) {
        use winit::Event::*;

        let mut running = true;
        while running {
            self.events_loop.poll_events(|event| match event {
                WindowEvent {
                    event: winit::WindowEvent::CloseRequested,
                    ..
                } => running = false,
                _ => (),
            });
        }
    }
}

/// Builder for application
///
/// Usage:
///
/// ```
/// let app = AppBuilder::new()
///     .size(640, 480)
///     .title("Example App")
///     .build()
///     .unwrap();
/// ```
pub struct AppBuilder {
    size: [u32; 2],
    title: &'static str,
}

impl AppBuilder {
    pub fn new() -> Self {
        AppBuilder {
            size: [640, 480],
            title: "rengine",
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

    /// Consumes the builder and creates the application
    pub fn build(self) -> Result<App, Box<dyn Error>> {
        let events_loop = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_title(self.title)
            .with_dimensions((self.size[0], self.size[1]).into())
            .build(&events_loop)?;

        Ok(App::new(events_loop, window))
    }
}
