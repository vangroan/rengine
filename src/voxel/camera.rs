//! Cast rays out of camera

use crate::camera::{ActiveCamera, CameraProjection, CameraView};
use crate::option::lift2;
use crate::res::DeviceDimensions;
use crate::voxel::{voxel_raycast, VoxelRaycast};
use glutin::dpi::{PhysicalPosition, PhysicalSize};
use nalgebra::{Matrix4, Perspective3, Point3, Unit};
use specs::{Read, ReadStorage};

/// Raycast from camera using system data
///
/// ## Example
///
/// ```ignore
/// let raycast = raycast_from_camera(world.system_data(), PhysicalPosition::new(800, 600), 1000)
/// ```
pub fn raycast_from_camera(
    data: (
        Read<'_, ActiveCamera>,
        Read<'_, DeviceDimensions>,
        ReadStorage<'_, CameraView>,
        ReadStorage<'_, CameraProjection>,
    ),
    screen_pos: PhysicalPosition,
    steps: u32,
) -> Option<VoxelRaycast> {
    let (active_camera, device_dim, cam_views, cam_projs) = data;

    let maybe_cam = active_camera
        .camera_entity()
        .and_then(|e| lift2(cam_projs.get(e), cam_views.get(e)));

    if let Some((cam_proj, cam_view)) = maybe_cam {
        // Build perspective projection that matches camera
        let projection = {
            let persp_settings = cam_proj.perspective_settings();
            Perspective3::new(
                persp_settings.aspect_ratio(),
                persp_settings.fovy().as_radians(),
                persp_settings.nearz(),
                persp_settings.farz(),
            )
        };

        return camera_raycast(
            projection,
            cam_view.view_matrix(),
            device_dim.physical_size().clone(),
            screen_pos,
            steps,
        );
    }

    None
}

pub fn camera_raycast(
    projection: Perspective3<f32>,
    view_matrix: Matrix4<f32>,
    device_size: PhysicalSize,
    screen_pos: PhysicalPosition,
    steps: u32,
) -> Option<VoxelRaycast> {
    // Point must be between [0.0, 1.0] to unproject
    let (device_w, device_h) = (device_size.width as f32, device_size.height as f32);

    // Convert glutin screen position to computer graphics screen coordinates
    let (screen_w, screen_h) = (
        screen_pos.x as f32 - (device_w / 2.),
        -(screen_pos.y as f32 - (device_h / 2.)),
    );

    // Use screen position to compute two points in clip space, where near
    // and far are -1 and 1 respectively.
    //
    // "ndc" = normalized device coordinates
    //
    // Multiplying with 2 is required because dividing the screen position
    // with the device size yields a value between 0.0 and 1.0. Normalized
    // device coordinates are a double unit cube, meaning each axis has a
    // range between -1.0 and 1.0.
    let near_ndc_point = Point3::new(
        (screen_w / device_w) * 2.0,
        (screen_h / device_h) * 2.0,
        -1.0,
    );
    let far_ndc_point = Point3::new(
        (screen_w / device_w) * 2.0,
        (screen_h / device_h) * 2.0,
        1.0,
    );

    // Unproject clip space points to view space
    let near_view_point = projection.unproject_point(&near_ndc_point);
    let far_view_point = projection.unproject_point(&far_ndc_point);

    // Compute line in view space
    let line_point = near_view_point;
    let line_direction = Unit::new_normalize(far_view_point - near_view_point);

    // Transform line from local camera space to world space
    let inverse_view_mat = view_matrix
        .try_inverse()
        .expect("Failed to compute inverse of view matrix");

    // Inverse matrix to transform device space to world space
    let world_point = inverse_view_mat.transform_point(&line_point);
    let world_direction = Unit::new_normalize(inverse_view_mat.transform_vector(&line_direction));

    // Create ray walker
    Some(voxel_raycast(world_point, world_direction, steps))
}
