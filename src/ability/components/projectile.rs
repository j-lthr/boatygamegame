use std::time::Duration;

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::ability::CastInfo;
use crate::common;
use crate::common::Faction;
use crate::event;
use crate::fx;

// Component for projectiles
#[derive(Component, Clone)]
pub struct Projectile {
    pub speed: f32,
    pub lifetime: f32,
    pub damage: i32,
}

pub fn collide(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Projectile, &CastInfo)>,
    target_query: Query<(Entity, &mut Transform, &common::Living), Without<Projectile>>,
    mut shoot_sounds: ResMut<Assets<fx::fm::FMSound>>,
    mut damage_events: EventWriter<event::DamageEvent>,
    faction_query: Query<&Faction>,
) {
    for (projectile_entity, projectile_transform, projectile, cast_info) in projectile_query {
        for (entity, target_transform, _living_opt) in &target_query {
            // Skip self-damage
            if entity == cast_info.caster {
                continue;
            }

            if let (Ok(source_faction), Ok(target_faction)) = (
                faction_query.get(cast_info.caster),
                faction_query.get(entity),
            ) {
                if source_faction == target_faction {
                    continue;
                }
            }

            let distance = projectile_transform
                .translation
                .distance(target_transform.translation);

            if distance < 1.0 {

                commands.entity(projectile_entity).despawn();
   
                // Apply damage if target has Living component

                let projectile_velocity = projectile_transform.forward() * projectile.speed;

                // Emit damage event instead of directly modifying health
                damage_events.write(event::DamageEvent {
                    target: entity,
                    source: Some(cast_info.caster),
                    damage: projectile.damage,
                    position: target_transform.translation,
                    impact_velocity: Some(projectile_velocity),
                });

                let shoot_sound_handle = shoot_sounds.add(fx::fm::FMSound {
                    config: fx::fm::HIT_SOUND,
                    duration: Duration::from_millis(100),
                });

                commands.spawn((
                    AudioPlayer(shoot_sound_handle),
                    PlaybackSettings::DESPAWN
                        .with_spatial(true)
                        .with_volume(Volume::Decibels(12.0)),
                    Transform::from_translation(target_transform.translation),
                ));

                break;
            }
        }
    }
}

/// System to move projectiles
pub fn handle_movement(
    mut projectile_query: Query<(&mut Transform, &Projectile)>,
    time: Res<Time>,
) {
    for (mut transform, projectile) in &mut projectile_query {
        let fwd = transform.forward();
        transform.translation += fwd * projectile.speed * time.delta_secs();
    }
}

/// System to cleanup old projectiles
pub fn cleanup(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile)>,
    time: Res<Time>,
) {
    for (entity, mut projectile) in &mut projectile_query {
        projectile.lifetime -= time.delta_secs();
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}


pub fn plugin(app: &mut bevy::app::App) {
    app.add_systems(
        Update,
        (
            collide,
            handle_movement,
            cleanup,
        )
    );
}