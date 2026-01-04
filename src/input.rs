use bevy::prelude::*;

use crate::{
    init::{DespawnOnReset, GameInit},
    player::Player,
};

#[derive(Component)]
pub struct Cursor;

pub fn update_cursor(
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    player: Query<&mut Transform, (With<Player>, Without<Cursor>)>,
    mut cursor: Query<(&mut Transform, &mut Visibility), With<Cursor>>,
    time: Res<Time>,
) -> Result {
    let window = windows.single()?;

    let (mut cursor_transform, mut visibility) = cursor.single_mut()?;

    if let Ok(cursor_pos) = window.cursor_position().ok_or("invalid cursor position") {
        let (camera, camera_transform) = camera.single()?;

        let ray = camera.viewport_to_world(camera_transform, cursor_pos)?;

        let player = player.single()?;

        if let Some(position) = ray
            .intersect_plane(Vec3::ZERO, InfinitePlane3d { normal: Dir3::Y })
            .map(|intersection| ray.origin + ray.direction * intersection)
        {
            cursor_transform.translation = position.with_y(player.translation.y);

            let target_transform = cursor_transform.looking_at(player.translation, Vec3::Y);
            cursor_transform.rotation = cursor_transform
                .rotation
                .slerp(target_transform.rotation, 10.0 * time.delta_secs());

            *visibility = Visibility::Visible;
        }
    } else {
        *visibility = Visibility::Hidden;
    }

    Ok(())
}

pub fn spawn_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut window: Query<&mut Window>,
) -> Result {
    commands.spawn((
        Cursor,
        DespawnOnReset,
        Transform::from_scale(Vec3::splat(0.7)),
        Mesh3d(meshes.add(Triangle3d::new(
            Vec3::new(0.5, 0.0, -0.5),
            Vec3::new(-0.5, 0.0, -0.5),
            Vec3::new(0.0, 0.0, 0.5),
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(10.0, 12.0, 20.0),
            ..default()
        })),
    ));

    window.single_mut()?.cursor_options.visible = false;

    Ok(())
}

pub fn plugin(app: &mut App) {
    app.add_systems(GameInit, spawn_cursor);
    app.add_systems(Update, update_cursor);
}
