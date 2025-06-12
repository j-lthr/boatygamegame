use crate::{event, init::DespawnOnReset};
use bevy::prelude::*;

#[derive(Component)]
pub struct DamageNumber {
    pub lifetime: Timer,
    pub initial_position: Vec3,
    pub rise_speed: f32,
}

pub(crate) struct DamageNumbersPlugin;

impl Plugin for DamageNumbersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_damage_numbers,
                update_damage_numbers,
                cleanup_damage_numbers,
            ),
        );
    }
}

pub fn spawn_damage_numbers(
    mut commands: Commands,
    mut damage_events: EventReader<event::DamageEvent>,
    asset_server: Res<AssetServer>,
) {
    for damage_event in damage_events.read() {
        // Spawn floating damage number
        let font = asset_server.load("fonts/Jersey15-Regular.ttf");

        // Different colors for different damage sources
        let color = match damage_event.source {
            Some(_) => Color::srgba(1.0, 1.0, 1.0, 0.0), // Red for player/enemy damage
            None => Color::srgb(1.0, 1.0, 0.3),          // Yellow for environmental damage
        };

        commands.spawn((
            Text2d::new(format!("-{}", damage_event.damage)),
            TextFont {
                font,
                font_size: 24.0,
                ..default()
            },
            TextColor(color),
            Transform::from_translation(damage_event.position + Vec3::new(0.0, 1.0, 0.0)),
            DamageNumber {
                lifetime: Timer::from_seconds(1.5, TimerMode::Once),
                initial_position: damage_event.position,
                rise_speed: 20.0,
            },
            DespawnOnReset
        ));
    }
}

pub fn update_damage_numbers(
    mut damage_number_query: Query<(&mut Transform, &mut DamageNumber, &mut TextColor)>,
    camera_3d_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    camera_2d_query: Query<(&GlobalTransform, &Camera), With<Camera2d>>,
    time: Res<Time>,
) {
    for (mut transform, mut damage_number, mut text_color) in &mut damage_number_query {
        damage_number.lifetime.tick(time.delta());

        // Calculate fade based on lifetime
        let progress =
            damage_number.lifetime.elapsed_secs() / damage_number.lifetime.duration().as_secs_f32();
        let alpha = 1.0 - progress;

        // Update alpha
        text_color.0.set_alpha(alpha);

        // Calculate world position (rising up)
        let world_pos = damage_number.initial_position
            + Vec3::new(0.0, damage_number.rise_speed * progress, 0.0);

        // Convert world position to screen position
        if let (Ok((camera_3d_transform, camera_3d)), Ok((camera_2d_transform, camera_2d))) =
            (camera_3d_query.single(), camera_2d_query.single())
        {
            if let Ok(viewport_position) =
                camera_3d.world_to_viewport(camera_3d_transform, world_pos)
            {
                if let Ok(world_pos_2d) =
                    camera_2d.viewport_to_world_2d(camera_2d_transform, viewport_position)
                {
                    transform.translation = Vec3::new(world_pos_2d.x, world_pos_2d.y, 0.0);
                    transform.scale = Vec3::splat(1.0 / (1.0 - progress))
                }
            }
        }
    }
}

pub fn cleanup_damage_numbers(
    mut commands: Commands,
    damage_number_query: Query<(Entity, &DamageNumber)>,
) {
    for (entity, damage_number) in &damage_number_query {
        if damage_number.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
