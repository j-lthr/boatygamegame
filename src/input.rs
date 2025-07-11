use bevy::prelude::*;

use crate::init::{DespawnOnReset, GameInit};

#[derive(Component)]
pub struct Cursor;


pub fn update_cursor(
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut cursor: Query<&mut Transform, With<Cursor>>,
) -> Result {
    let window = windows.single()?;
    let cursor_pos = window.cursor_position().ok_or("invalid cursor position")?;

    let (camera, camera_transform) = camera.single()?;
    let mut cursor_transform = cursor.single_mut()?;

    let ray = camera.viewport_to_world(camera_transform, cursor_pos)?;

    if let Some(position) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d { normal: Dir3::Y })
        .map(|intersection| ray.origin + ray.direction * intersection) {
            cursor_transform.translation = position;
    }

    Ok(())
}

pub fn spawn_cursor(    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>) {
        commands.spawn((
            Cursor,
            DespawnOnReset,
            Transform::default(),
            Mesh3d(meshes.add(Sphere::new(0.2))),
            MeshMaterial3d(materials.add(StandardMaterial {
                emissive: LinearRgba::rgb(10.0, 10.0, 10.0),
                ..default()
            })),
        ));
    }

pub fn plugin(app: &mut App) {
    app.add_systems(GameInit, spawn_cursor);
    app.add_systems(Update, update_cursor);
}

