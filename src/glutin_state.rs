use glutin::{EventsLoop, Window};

/// Wrapper for Glutin objects
#[derive(Debug)]
pub struct GlutinState {
    events_loop: EventsLoop,
    window: Window,
}

impl GlutinState {
    pub fn new(events_loop: EventsLoop, window: Window) -> Self {
        GlutinState {
            events_loop,
            window,
        }
    }

    #[inline]
    pub fn events_loop(&self) -> &EventsLoop {
        &self.events_loop
    }

    #[inline]
    pub fn events_loop_mut(&mut self) -> &mut EventsLoop {
        &mut self.events_loop
    }

    #[inline]
    pub fn window(&self) -> &Window {
        &self.window
    }

    #[inline]
    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}
