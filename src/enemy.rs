use std::f32::consts::PI;
use std::time::Duration;

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::event;
use crate::player;
use crate::procedural;
use crate::fx;
use crate::common;
use crate::state;

// Component for enemies
#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
    pub jitter: f32,
}

#[derive(Component)]
pub struct SlamAttacker {
    pub slam_timer: Timer,
    pub slam_range: f32,
    pub slam_damage: i32,
}

#[derive(Component)]
pub struct SlamFadeEffect {
    pub radius: f32,
    pub alpha: f32,
}


#[derive(Resource)]
pub struct EnemySpawnTimer {
    pub timer: Timer,
}

/// System to spawn enemies
pub fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    time: Res<Time>,
    player_query: Query<&Transform, With<player::Player>>,
    mut spawn_event_writer: EventWriter<event::SpawnEvent>,
) {
    spawn_timer.timer.tick(time.delta());

    if spawn_timer.timer.just_finished() {
        if let Ok(player_transform) = player_query.single() {
            // Spawn enemy at random position around the player
            let angle = fastrand::f32() * 2.0 * PI;
            let distance = 10.0 + fastrand::f32() * 5.0; // 10-15 units away
            let spawn_pos = Vec3::new(
                player_transform.translation.x + angle.cos() * distance,
                0.75, // Ground level (half of capsule height)
                player_transform.translation.z + angle.sin() * distance,
            );

            let mesh = meshes.add(procedural::rock::generate_rock_mesh(&procedural::rock::RockConfig::default()));

            let enemy = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(10.0, 5.0, 0.0),
                        emissive: Color::srgb(4.0, 2.0, 0.0).into(), // Slightly glowing
                        ..default()
                    })),
                    Transform::from_translation(spawn_pos),
                    Enemy {
                        speed: 5.0 + fastrand::f32() * 0.2,
                        jitter: 0.1, //0.1 + fastrand::f32() * 0.2, // Random jitter between 0.1 and 0.3
                    },
                    SlamAttacker {
                        slam_timer: Timer::from_seconds(0.25, TimerMode::Once),
                        slam_range: 3.0,
                        slam_damage: 33,
                    },
                    common::Living {
                        health: 8,
                        max_health: 8,
                    },
                ))
                .id();

            spawn_event_writer.write(event::SpawnEvent { entity: enemy });
        }
    }
}

/// System to move enemies toward player with rolling motion
pub fn move_enemies(
    mut enemy_query: Query<(&mut Transform, &Enemy)>,
    player_query: Query<&Transform, (With<player::Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut enemy_transform, enemy) in &mut enemy_query {
            // Calculate direction to player
            let direction =
                (player_transform.translation - enemy_transform.translation).normalize_or_zero();
            let horizontal_direction = Vec3::new(direction.x, 0.0, direction.z).normalize_or_zero();

            // Calculate movement for this frame
            let movement_distance = enemy.speed * time.delta_secs();
            let movement_vector = horizontal_direction * movement_distance;

            // Move enemy toward player
            enemy_transform.translation += movement_vector;

            enemy_transform.translation.x += enemy.jitter * (fastrand::f32() - 0.5); // Add slight random jitter
            enemy_transform.translation.z += enemy.jitter * (fastrand::f32() - 0.5); // Add slight random jitter

            // Add rolling motion
            if horizontal_direction.length() > 0.0 {
                // Assume boulder radius for rolling calculation (adjust as needed)
                let boulder_radius = 0.5; // Adjust this based on your boulder size

                // Calculate rotation angles based on movement
                let roll_angle = movement_vector.x / boulder_radius; // Roll around Z-axis for X movement
                let pitch_angle = -movement_vector.z / boulder_radius; // Pitch around X-axis for Z movement (negative for correct direction)

                // Apply rolling rotation
                let roll_rotation = Quat::from_rotation_z(roll_angle);
                let pitch_rotation = Quat::from_rotation_x(pitch_angle);

                // Combine rotations and apply to current rotation
                enemy_transform.rotation =
                    enemy_transform.rotation * roll_rotation * pitch_rotation;
            }
        }
    }
}


// Updated enemy_slam_attack system to trigger game over
pub fn enemy_slam_attack(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shoot_sounds: ResMut<Assets<fx::fm::FMSound>>,
    time: Res<Time>,
    mut enemy_query: Query<(&mut Transform, &mut Enemy, &mut SlamAttacker)>,
    mut player_query: Query<(&mut Transform, &mut common::Living), (With<player::Player>, Without<Enemy>)>,
    mut next_state: ResMut<NextState<state::GameState>>,
) {
    if let Ok((mut player_transform, mut player_living)) = player_query.single_mut() {
        for (mut enemy_transform, mut enemy, mut slam_attacker) in &mut enemy_query {
            slam_attacker.slam_timer.tick(time.delta());

            let distance = enemy_transform
                .translation
                .distance(player_transform.translation);

            if distance <= slam_attacker.slam_range && slam_attacker.slam_timer.finished() {
                slam_attacker.slam_timer.reset();
                player_living.health -= slam_attacker.slam_damage;

                let knockback_vector = (player_transform.translation - enemy_transform.translation)
                    .normalize_or_zero()
                    .with_y(0.0)
                    * slam_attacker.slam_range
                    * 1.0;

                player_transform.translation += knockback_vector;

                spawn_slam_effect(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    enemy_transform.translation,
                    slam_attacker.slam_range,
                );

                let slam_sound_handle = shoot_sounds.add(fx::fm::FMSound {
                    config: fx::fm::SLAM,
                    duration: Duration::from_millis(400),
                });

                commands.spawn((
                    AudioPlayer(slam_sound_handle),
                    PlaybackSettings::DESPAWN
                        .with_spatial(true)
                        .with_volume(Volume::Decibels(18.0)),
                    Transform::from_translation(enemy_transform.translation),
                ));

                println!(
                    "Player hit by slam attack! Health: {}",
                    player_living.health
                );

                // Check if player is dead
                if player_living.health <= 0 {
                    println!("Game Over!");
                    next_state.set(state::GameState::GameOver);
                }
            }
        }
    }
}

// New function to spawn the circular slam effect
pub fn spawn_slam_effect(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
) {
    // Create a circular mesh (torus or cylinder would work, but let's use a thin cylinder)
    let circle_mesh = meshes.add(Cylinder::new(2.0, 0.1)); // radius 2.0, height 0.1

    // Create a material with high emission and transparency
    let slam_material = materials.add(StandardMaterial {
        base_color: Color::srgba(2.0, 1.0, 1.0, 0.8), // Orange-red with transparency
        emissive: Color::srgb(2.0, 1.0, 1.0).into(),  // Bright orange glow
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Spawn the slam effect slightly above ground
    commands.spawn((
        Mesh3d(circle_mesh),
        MeshMaterial3d(slam_material),
        Transform::from_translation(position + Vec3::new(0.0, 0.05, 0.0)),
        SlamFadeEffect {
            radius: radius * 1.1,
            alpha: 1.0, // Start fully visible
        },
    ));
}

// New system to animate and cleanup slam effects
pub fn animate_fade_effects(
    mut commands: Commands,
    mut slam_effect_query: Query<(Entity, &mut Transform, &mut SlamFadeEffect)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut slam_effect) in &mut slam_effect_query {
        // Fade duration - effect lasts for 0.5 seconds
        let fade_speed = 10.0; // Higher = faster fade
        slam_effect.alpha -= fade_speed * time.delta_secs();

        // Expand the circle slightly as it fades
        slam_effect.radius += 3.0 * time.delta_secs(); // Expand at 3 units per second
        transform.scale = Vec3::splat(slam_effect.radius / 2.0); // Scale based on radius

        // Update material alpha if we can access it
        if let Ok(material_handle) = material_query.get(entity) {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                // Update both base color alpha and emissive intensity
                material.base_color.set_alpha(slam_effect.alpha.max(0.0));
                let emissive_intensity = slam_effect.alpha.max(0.0) * 5.0; // Scale emissive with alpha
                material.emissive =
                    Color::srgb(emissive_intensity, emissive_intensity * 0.3, 0.0).into();
            }
        }

        // Remove effect when fully faded
        if slam_effect.alpha <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
