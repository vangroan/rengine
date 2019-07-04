use crate::angle::Rad;
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
}

/// Builder methods that consume the `Transform` and returns it
impl Transform {
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

    /// Rotates around given axis in local space, by an angle expressed as radians
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
    ///         .with_rotate(angle, Z_AXIS);
    /// }
    /// ```
    #[inline]
    pub fn with_rotate<A, V>(mut self, angle: A, axis: V) -> Self
    where
        A: Into<Rad<f32>>,
        V: Into<Vec3>,
    {
        self.rotate(angle, axis);
        self
    }

    #[inline]
    pub fn with_rotation<A, V>(mut self, angle: A, axis: V) -> Self
    where
        A: Into<Rad<f32>>,
        V: Into<Vec3>,
    {
        self.set_rotation(angle, axis);
        self
    }

    #[inline]
    pub fn with_rotate_world<A, V>(mut self, angle: A, axis: V) -> Self
    where
        A: Into<Rad<f32>>,
        V: Into<Vec3>,
    {
        self.rotate_world(angle, axis);
        self
    }
}

/// Methods that mutate the `Transform `in place
impl Transform {
    #[inline]
    pub fn translate<V>(&mut self, offset: V)
    where
        V: Into<Vec3>,
    {
        self.pos += offset.into();
    }

    #[inline]
    pub fn rotate<A, V>(&mut self, angle: A, axis: V)
    where
        A: Into<Rad<f32>>,
        V: Into<Vec3>,
    {
        self.rot = glm::quat_rotate(&self.rot, angle.into().as_radians(), &axis.into());
    }

    #[inline]
    pub fn set_rotation<A, V>(&mut self, angle: A, axis: V)
    where
        A: Into<Rad<f32>>,
        V: Into<Vec3>,
    {
        self.rot = glm::quat_rotate(
            &Qua::<f32>::identity(),
            angle.into().as_radians(),
            &axis.into(),
        );
    }

    #[inline]
    pub fn rotate_world<A, V>(&mut self, angle: A, axis: V)
    where
        A: Into<Rad<f32>>,
        V: Into<Vec3>,
    {
        let world_rot = glm::quat_rotate(
            &Qua::<f32>::identity(),
            angle.into().as_radians(),
            &axis.into(),
        );
        self.rot = world_rot * self.rot;
    }

    /// Orient the transformation towards the given
    /// position in local space.
    #[inline]
    pub fn look_at<V>(&mut self, direction: V, up: V)
    where
        V: Into<Vec3>,
    {
        // FIXME: This solution works for now. Later we must look
        //        into simply creating the look at matrix ourselves
        //        so it's correct immediately.
        //        Currently we have to point the direction away from
        //        the target, and invesrse the quaternion to avoid
        //        flipping the axese.

        // Look at matrix will orient the object away from
        // the camera. We point the direction away from the
        // target for correct orientation.
        let backward = direction.into() * -1.0;
        let lookat = glm::quat_look_at_rh(&backward, &up.into());

        // look_at is designed for cameras, which move the
        // entire world, keeping the camera at the origin.
        //
        // Inverse is required because we're not transforming
        // the camera, but objects in the world.
        if let Some(inverse_quat) = lookat.try_inverse() {
            // Inverse is None when Quaternion is zero
            self.rot = inverse_quat;
        }
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
