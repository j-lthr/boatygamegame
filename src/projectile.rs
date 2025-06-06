use std::time::Duration;

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::common;
use crate::event;
use crate::fx;
use crate::state::GameScore;

// Component for projectiles
#[derive(Component)]
pub struct Projectile {
    pub direction: Vec3,
    pub speed: f32,
    pub lifetime: f32,
    pub damage: i32,
    pub source: Entity, // Entity that fired this projectile
}


pub fn collide(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    projectile_query: Query<(Entity, &Transform, &Projectile)>,
    mut target_query: Query<(Entity, &mut Transform, &common::Living), Without<Projectile>>,
    mut shoot_sounds: ResMut<Assets<fx::fm::FMSound>>,
    blood_materials: Res<fx::blood::BloodMaterials>,
    mut damage_events: EventWriter<event::DamageEvent>,
) {
    for (projectile_entity, projectile_transform, projectile) in &projectile_query {
        for (entity, mut target_transform, living_opt) in &mut target_query {
            // Skip self-damage
            if entity == projectile.source {
                continue;
            }

            let distance = projectile_transform
                .translation
                .distance(target_transform.translation);

            if distance < 0.7 {
                // Hit detection radius
                commands.entity(projectile_entity).despawn();

                // Apply damage if target has Living component
            
                // Emit damage event instead of directly modifying health
                damage_events.write(event::DamageEvent {
                    target: entity,
                    source: Some(projectile.source),
                    damage: projectile.damage,
                    position: target_transform.translation,
                });

                let projectile_velocity = projectile.direction * projectile.speed;

                fx::blood::spawn_blood_explosion(
                    &mut commands,
                    &mut meshes,
                    projectile_transform.translation,
                    &blood_materials,
                    15,
                    2.5,
                    -projectile_velocity * 0.1,
                    Vec3::new(1.0, 0.5, 0.0),
                );

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

                target_transform.translation += projectile_velocity.with_y(0.0) * 0.01;
                
                break;
            }
        }
    }
}



/// System to move projectiles
pub fn handle_movement(mut projectile_query: Query<(&mut Transform, &Projectile)>, time: Res<Time>) {
    for (mut transform, projectile) in &mut projectile_query {
        transform.translation += projectile.direction * projectile.speed * time.delta_secs();
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


