use glm::{Mat4x4, Qua, Vec3};
use specs::{Component, DenseVecStorage};

pub const X_AXIS: [f32; 3] = [1.0, 0.0, 0.0];
pub const Y_AXIS: [f32; 3] = [0.0, 1.0, 0.0];
pub const Z_AXIS: [f32; 3] = [0.0, 0.0, 1.0];

#[derive(Component, Debug)]
#[storage(DenseVecStorage)]
pub struct Transform {
    pub(crate) anchor: Vec3,
    pub(crate) pos: Vec3,
    pub(crate) scale: Vec3,
    pub(crate) rot: Qua<f32>,
}

impl Transform {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn matrix(&self) -> Mat4x4 {
        let mut m = Mat4x4::identity();

        m = glm::translate(&m, &self.pos);
        m = m * nalgebra_glm::quat_to_mat4(&self.rot);
        m = glm::scale(&m, &self.scale);
        m = glm::translate(&m, &(-self.anchor));

        m
    }

    #[inline]
    pub fn with_anchor<V>(mut self, anchor: V) -> Self
    where
        V: Into<Vec3>,
    {
        self.anchor = anchor.into();
        self
    }

    #[inline]
    pub fn with_position<V>(mut self, position: V) -> Self
    where
        V: Into<Vec3>,
    {
        self.pos = position.into();
        self
    }

    #[inline]
    pub fn with_scale<V>(mut self, scale: V) -> Self
    where
        V: Into<Vec3>,
    {
        self.scale = scale.into();
        self
    }

    /// Rotate around given axis, by an angle expressed as radians
    ///
    /// ```
    /// extern crate rengine;
    /// use rengine::comp::{Transform, Z_AXIS};
    /// use std::f32::consts::PI;
    ///
    /// fn main() {
    ///     // 45 degrees
    ///     let angle = 45. * (PI / 180.);
    ///
    ///     let mut t = Transform::new()
    ///         .with_rotation(angle, Z_AXIS);
    /// }
    /// ```
    #[inline]
    pub fn with_rotation<V>(mut self, angle: f32, axis: V) -> Self
    where
        V: Into<Vec3>,
    {
        self.rot = glm::quat_rotate(&self.rot, angle, &axis.into());
        self
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            anchor: Vec3::new(0., 0., 0.),
            pos: Vec3::new(0., 0., 0.),
            scale: Vec3::new(1., 1., 1.),
            rot: Qua::<f32>::identity(),
        }
    }
}
