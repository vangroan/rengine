use std::time::Duration;

/// The time it took for the last frame to elapse
#[derive(Default, Clone)]
pub struct DeltaTime(pub(crate) Duration);

impl DeltaTime {
    #[inline]
    pub fn duration(&self) -> &Duration {
        &self.0
    }
}
