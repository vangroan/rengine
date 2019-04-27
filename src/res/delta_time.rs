use std::time::Duration;

/// The time it took for the last frame to elapse
#[derive(Default, Clone)]
pub struct DeltaTime(pub(crate) Duration);

impl DeltaTime {
    #[inline]
    pub fn duration(&self) -> &Duration {
        &self.0
    }

    #[inline]
    pub fn as_secs_float(&self) -> f32 {
        self.0.as_millis() as f32 / 1000.
    }
}
