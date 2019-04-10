use glm::DVec3;
use specs::{Component, DenseVecStorage};

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Transform {
    pub(crate) pos: DVec3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            pos: DVec3::new(0., 0., 0.),
        }
    }
}
