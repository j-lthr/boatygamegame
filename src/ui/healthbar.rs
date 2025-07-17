use bevy::prelude::*;

use crate::common;
use crate::event;

#[derive(Component)]
pub struct HealthBar {
    pub entity: Entity,
}

pub(crate) struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn, update, clean_up));
    }
}

pub fn spawn(
    mut commands: Commands,
    mut spawn_event_reader: EventReader<event::SpawnEvent>,
    mut mesh_server: ResMut<Assets<Mesh>>,
    mut material_server: ResMut<Assets<ColorMaterial>>,
) {
    for event in spawn_event_reader.read() {
        commands.spawn((
            MeshMaterial2d(material_server.add(ColorMaterial {
                color: Color::srgb(1.0, 1.0, 1.0),
                ..Default::default()
            })),
            Mesh2d(mesh_server.add(Rectangle::from_size(Vec2::new(48.0, 8.0)))),
            Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(0.5)),
            HealthBar {
                entity: event.entity,
            },
        ));
    }
}

pub fn update(
    mut health_bar_query: Query<(&HealthBar, &mut Transform)>,
    living_query: Query<(&common::HealthPool, &GlobalTransform)>,
    camera_3d_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    camera_2d_query: Query<(&GlobalTransform, &Camera), With<Camera2d>>,
) {
    for (health_bar, mut healthbar_transform) in &mut health_bar_query {
        if let Ok((living, enemy_transform)) = living_query.get(health_bar.entity) {
            healthbar_transform.scale.x = living.health_fraction();

            if let (Ok((camera_3d_transform, camera_3d)), Ok((camera_2d_transform, camera_2d))) =
                (camera_3d_query.single(), camera_2d_query.single())
            {
                let _ = camera_3d
                    .world_to_viewport(
                        camera_3d_transform,
                        enemy_transform.translation() + Vec3::Z * 1.4,
                    )
                    .map(|viewport_position| {
                        camera_2d
                            .viewport_to_world_2d(camera_2d_transform, viewport_position)
                            .map(|world_pos_2d| {
                                // Update health bar position in 2D space
                                healthbar_transform.translation =
                                    Vec3::new(world_pos_2d.x, world_pos_2d.y, 0.0);
                            })
                    });
            }
        }
    }
}

pub fn clean_up(
    mut commands: Commands,
    health_bar_query: Query<(Entity, &HealthBar)>,
    enemy_query: Query<()>,
) {
    for (health_bar_entity, health_bar) in &health_bar_query {
        // If the enemy no longer exists, remove its health bar
        if enemy_query.get(health_bar.entity).is_err() {
            commands.entity(health_bar_entity).despawn();
        }
    }
}
