use std::f32;
use std::f32::consts::PI;
use bevy::prelude::*;

use crate::common::Living;
use crate::event;
use crate::player;
use crate::procedural;
use crate::common;
use crate::ability::slam::{Slam, SlamParams};
use crate::ability::shotgun::{Shotgun, ShotgunParams};
use crate::ability::dash::{Dash, DashParams};
use crate::ability::{AbilitySlot, AttemptCastEvent};
use crate::utils::normal_dist_1d;

// Component for enemies
#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
    pub jitter: f32,
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
            
            let pack_size = fastrand::i32(2..4);

            for i in 0..pack_size {

            let offset_angle = i as f32 / pack_size as f32 * f32::consts::PI * 2.0;

            let enemy = commands
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(10.0, 5.0, 0.0),
                        emissive: Color::srgb(4.0, 2.0, 0.0).into(), // Slightly glowing
                        ..default()
                    })),
                    Transform::from_translation(spawn_pos + vec3(f32::cos(offset_angle),0.0, f32::sin(offset_angle))),
                    Enemy {
                        speed: 5.0 + fastrand::f32() * 0.2,
                        jitter: 0.1, //0.1 + fastrand::f32() * 0.2, // Random jitter between 0.1 and 0.3
                    },
                    AbilitySlot {
                        cooldown: Timer::from_seconds(0.25, TimerMode::Once),
                        name: "Slam",
                        ability: Slam {
                            range: normal_dist_1d(3.0, 0.5),
                            damage: 33,
                            knockback_force: 3.0,
                        }
                    },
                    AbilitySlot {
                        cooldown: Timer::from_seconds(normal_dist_1d(3.0, 0.5).abs(), TimerMode::Once),
                        name: "Shotgun",
                        ability: Shotgun {
                            bullet_count: 10,
                            spread: 2.0,
                            speed: 30.0,
                            lifetime: 1.0,
                            damage: 1,
                        }
                    },
                    AbilitySlot {
                        cooldown: Timer::from_seconds(3.0, TimerMode::Once),
                        name: "Dash",
                        ability: Dash {
                            range: 6.0,
                        }
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


pub fn enemy_combat_ai(
    mut enemy_query: Query<(Entity, &Transform, &AbilitySlot<Slam>, &AbilitySlot<Shotgun>, &AbilitySlot<Dash>, &Living, &mut Enemy)>,
    player_query: Query<&Transform, (With<player::Player>, Without<Enemy>)>,
    mut slam_action: EventWriter<AttemptCastEvent<Slam>>,
    mut shotgun_action: EventWriter<AttemptCastEvent<Shotgun>>,
    mut dash_action: EventWriter<AttemptCastEvent<Dash>>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (enemy_entity, enemy_transform, slam_ability, _shotgun_ability, dash_ability, living, mut enemy) in &mut enemy_query {
            let distance = enemy_transform.translation.distance(player_transform.translation);

            let enraged = living.health_fraction() < 0.5;

            if enraged {
                enemy.speed = 7.0;
                enemy.jitter = 0.3;
            }
            
            // Use dash to close distance if far away (aggressive pursuit)
            if distance > 8.0 && enraged  {
                let direction = (player_transform.translation - enemy_transform.translation)
                    .normalize_or_zero()
                    .with_y(0.0);
                    
                dash_action.write(AttemptCastEvent {
                    caster: enemy_entity,
                    params: DashParams::Directional(direction),
                    _marker: std::marker::PhantomData::default(),
                });
            }

            // Use slam if in range
            if distance <= slam_ability.ability.range {
                slam_action.write(AttemptCastEvent {
                    caster: enemy_entity,
                    params: SlamParams {
                        target_position: player_transform.translation,
                    },
                    _marker: std::marker::PhantomData::default(),
                });
            }

            /*// Use shotgun if out of slam range but within shooting range
            if distance <= 30.0 { // Shooting range
                shotgun_action.write(AttemptCastEvent {
                    caster: enemy_entity,
                    params: ShotgunParams {
                        target_position: player_transform.translation,
                    },
                    _marker: std::marker::PhantomData::default(),
                });
            }*/
        }
    }
}

