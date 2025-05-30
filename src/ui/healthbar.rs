use bevy::prelude::*;

use crate::common;
use crate::event;

#[derive(Component)]
pub struct HealthBar {
    pub entity: Entity,
}

pub fn spawn(
    mut commands: Commands,
    mut spawn_event_reader: EventReader<event::SpawnEvent>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for event in spawn_event_reader.read() {
        // Spawn health bar above enemy
        let texture = asset_server.load("sprites/healthbar.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::new(64, 4), 1, 8, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        commands.spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 7,
                },
            ),
            Transform::from_xyz(0.0, 0.0, 0.0),
            HealthBar {
                entity: event.entity,
            },
        ));
    }
}


pub fn update(
    mut health_bar_query: Query<(&HealthBar, &mut Sprite, &mut Transform)>,
    living_query: Query<(&common::Living, &GlobalTransform)>,
    camera_3d_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    camera_2d_query: Query<(&GlobalTransform, &Camera), With<Camera2d>>,
) {
    for (health_bar, mut sprite, mut healthbar_transform) in &mut health_bar_query {
        if let Ok((living, enemy_transform)) = living_query.get(health_bar.entity) {
            // Update health bar position above enemy
            if let Some(ref mut texture_atlas) = sprite.texture_atlas {
                let index = ((7.0 * (living.health as f32 / living.max_health as f32)) as usize)
                    .max(0)
                    .min(7);
                texture_atlas.index = index; // Update based on health

                sprite.color.set_alpha(if index == 7 { 0.0 } else { 1.0 });
            }

            if let (Ok((camera_3d_transform, camera_3d)), Ok((camera_2d_transform, camera_2d))) =
                (camera_3d_query.single(), camera_2d_query.single())
            {
                let _ = camera_3d
                    .world_to_viewport(
                        camera_3d_transform,
                        enemy_transform.translation() + Vec3::Y * 1.5,
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