use specs::Entity;

/// Keeps a handle to the currently active camera, which
/// will be used for rendering.
///
/// When no camera entity is set, nothing will be rendered.
pub struct ActiveCamera(Option<Entity>);

impl ActiveCamera {
    pub fn new(entity: Entity) -> Self {
        ActiveCamera(Some(entity))
    }

    #[inline]
    pub fn camera_entity(&self) -> Option<Entity> {
        self.0.clone()
    }

    #[inline]
    pub fn set_camera_entity(&mut self, entity: Entity) {
        self.0 = Some(entity);
    }

    #[inline]
    pub fn clear_camera(&mut self) {
        self.0 = None;
    }
}

impl Default for ActiveCamera {
    fn default() -> Self {
        ActiveCamera(None)
    }
}
