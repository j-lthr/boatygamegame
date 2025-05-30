use std::time::Duration;

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::common;
use crate::fx;
use crate::state::GameScore;

// Component for bullets
#[derive(Component)]
pub struct Bullet {
    pub direction: Vec3,
    pub speed: f32,
    pub lifetime: f32,
}


pub fn collide<Marker: Component>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    bullet_query: Query<(Entity, &Transform, &Bullet), Without<Marker>>,
    mut enemy_query: Query<(Entity, &mut Transform, &mut common::Living), (With<Marker>, Without<Bullet>)>,
    mut shoot_sounds: ResMut<Assets<fx::fm::FMSound>>,
    blood_materials: Res<fx::blood::BloodMaterials>,
    mut score: ResMut<GameScore>,
) {
    for (bullet_entity, bullet_transform, bullet) in &bullet_query {
        for (entity, mut enemy_transform, mut living) in &mut enemy_query {
            let distance = bullet_transform
                .translation
                .distance(enemy_transform.translation);

            if distance < 0.7 {
                // Hit detection radius
                commands.entity(bullet_entity).despawn();

                // Damage enemy
                living.health -= 1;

                let bullet_velocity = bullet.direction * bullet.speed;

                fx::blood::spawn_blood_explosion(
                    &mut commands,
                    &mut meshes,
                    bullet_transform.translation,
                    &blood_materials,
                    15,
                    2.5,
                    -bullet_velocity * 0.1,
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
                    Transform::from_translation(enemy_transform.translation),
                ));

                if living.health <= 0 {
                    // Enemy killed - update score!
                    score.kills += 1;
                    score.combo += 1;
                    score.combo_timer.reset(); // Reset combo timer

                    // Calculate points with combo multiplier
                    let base_points = 100;
                    let combo_bonus = (score.combo - 1) * 50; // 50 extra points per combo level
                    let points_earned = base_points + combo_bonus;
                    score.current += points_earned;

                    fx::blood::spawn_blood_explosion(
                        &mut commands,
                        &mut meshes,
                        enemy_transform.translation,
                        &blood_materials,
                        100,
                        5.0,
                        Vec3::ZERO,
                        Vec3::new(10.0, 5.0, 0.0),
                    );

                    let shoot_sound_handle = shoot_sounds.add(fx::fm::FMSound {
                        config: fx::fm::DEATH_SOUND,
                        duration: Duration::from_millis(100),
                    });

                    commands.spawn((
                        AudioPlayer(shoot_sound_handle),
                        PlaybackSettings::DESPAWN
                            .with_spatial(true)
                            .with_volume(Volume::Decibels(36.0)),
                        Transform::from_translation(enemy_transform.translation),
                    ));

                    commands.entity(entity).despawn();
                }

                enemy_transform.translation += bullet_velocity.with_y(0.0) * 0.01;
                break;
            }
        }
    }
}



/// System to move bullets
pub fn handle_movement(mut bullet_query: Query<(&mut Transform, &Bullet)>, time: Res<Time>) {
    for (mut transform, bullet) in &mut bullet_query {
        transform.translation += bullet.direction * bullet.speed * time.delta_secs();
    }
}

/// System to cleanup old bullets
pub fn cleanup(
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &mut Bullet)>,
    time: Res<Time>,
) {
    for (entity, mut bullet) in &mut bullet_query {
        bullet.lifetime -= time.delta_secs();
        if bullet.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}


