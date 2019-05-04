use crate::render::DrawFactory;

pub struct GizmoDrawSystem {}

impl DrawFactory for GizmoDrawSystem {
    fn create() -> Self {
        GizmoDrawSystem {}
    }
}
