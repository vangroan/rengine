use glm::{Mat4x4, Quat, Vec3};
use specs::{Component, DenseVecStorage};

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Transform {
    pub(crate) pos: Vec3,
    pub(crate) scale: Vec3,
    pub(crate) rotation: Quat,
}

impl Transform {
    pub fn matrix(&self) -> Mat4x4 {
        let m = Mat4x4::identity();
        glm::translate(&m, &self.pos)
    }

    pub fn with_position<V>(mut self, position: V) -> Self
    where
        V: Into<Vec3>,
    {
        self.pos = position.into();
        self
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            pos: Vec3::new(0., 0., 0.),
            scale: Vec3::new(1., 1., 1.),
            rotation: Quat::identity(),
        }
    }
}
