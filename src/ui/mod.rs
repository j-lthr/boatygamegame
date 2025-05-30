use bevy::prelude::*;

pub mod healthbar;


pub fn compute_3d_cursor_pos(
    windows: Query<&Window>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec3> {
    let window = windows.single().ok()?;
    let cursor_pos = window.cursor_position()?;
    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;

    ray.intersect_plane(Vec3::ZERO, InfinitePlane3d { normal: Dir3::Y })
        .map(|intersection| ray.origin + ray.direction * intersection)
}