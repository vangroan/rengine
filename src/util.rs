//! Non-essential utilities

use specs::{Component, DenseVecStorage};
use std::fmt;

const FPS_COUNTER_WINDOW_SIZE: usize = 60;

#[derive(Component)]
pub struct FpsCounter {
    /// Sliding window of timer durations for
    /// past frames, stored as seconds.
    frames: [f32; FPS_COUNTER_WINDOW_SIZE],

    /// Current position in frame duration window
    cursor: usize,
}

impl FpsCounter {
    pub fn new() -> Self {
        FpsCounter::default()
    }

    /// Records the delta time for a frame.
    pub fn add(&mut self, duration: &::std::time::Duration) {
        let millis = duration.as_millis();
        self.frames[self.cursor] = millis as f32 / 1000.0;
        self.cursor = (self.cursor + 1) % FPS_COUNTER_WINDOW_SIZE;
    }

    /// Calculates the frames per second for the past
    /// frame window.
    pub fn fps(&self) -> f32 {
        if FPS_COUNTER_WINDOW_SIZE == 0 {
            panic!("FPS Counter window size is zero")
        }

        let total = self.frames.iter().fold(0.0, |acc, x| acc + x);
        let average_dt = total / FPS_COUNTER_WINDOW_SIZE as f32;

        if average_dt != 0.0 {
            1.0 / average_dt
        } else {
            0.0
        }
    }
}

impl Default for FpsCounter {
    fn default() -> Self {
        FpsCounter {
            frames: [0.0; FPS_COUNTER_WINDOW_SIZE],
            cursor: 0,
        }
    }
}

impl fmt::Display for FpsCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FPS({})", self.fps())
    }
}

// ---------- //
// Gui Widget //
// ---------- //

use crate::colors;
use crate::comp::Transform;
use crate::gui;
use specs::{Builder, Entity, World};

/// Helper to create a basic FPS counter text output.
///
/// The text will be added to the root widget.
pub fn create_fps_counter_widget(world: &mut World) -> Entity {
    let entity = world
        .create_entity()
        .with(gui::Placement::new(0.0, 0.0))
        .with(gui::GlobalPosition::default())
        .with(gui::BoundsRect::new(32.0, 32.0)) // TODO: Proper bounds based on behaviour of glyph_brush
        .with(gui::text::TextBatch::default().with("FPS: 0", colors::WHITE))
        .with(Transform::default())
        .build();

    let _node_id = world
        .write_resource::<gui::GuiGraph>()
        .insert_entity(entity, None);

    entity
}
