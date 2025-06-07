use std::f32;
use std::f32::consts::PI;
use bevy::prelude::*;

use crate::common::Faction;
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
use super::{Enemy, AttemptSpawnEvent};

pub const BOULDER_COLOR: Color = Color::srgb(4.0, 2.0, 4.0);

// Component for boulder enemies
#[derive(Component, Clone)]
pub struct Boulder {
    pub speed: f32,
    pub jitter: f32,
}

#[derive(Clone)]
pub struct BoulderSpawnParams {
    pub position: Vec3,
    pub pack_size: i32,
}

/// System to handle boulder spawn events
pub fn handle_boulder_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_attempts: EventReader<AttemptSpawnEvent<Boulder>>,
    mut spawn_event_writer: EventWriter<event::SpawnEvent>,
) {
    for spawn_attempt in spawn_attempts.read() {
        let mesh = meshes.add(procedural::rock::generate_rock_mesh(&procedural::rock::RockConfig::default()));
        
        for i in 0..spawn_attempt.params.pack_size {
            let offset_angle = i as f32 / spawn_attempt.params.pack_size as f32 * f32::consts::PI * 2.0;

            let enemy = commands
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(10.0, 5.0, 10.0),
                        emissive: BOULDER_COLOR.into(), // Slightly glowing
                        ..default()
                    })),
                    Transform::from_translation(spawn_attempt.params.position + vec3(f32::cos(offset_angle),0.0, f32::sin(offset_angle))),
                    Boulder {
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
                        cooldown: Timer::from_seconds(0.25, TimerMode::Once),
                        name: "Shotgun",
                        ability: Shotgun {
                            bullet_count: 1,
                            spread: 0.0,
                            speed: 30.0,
                            lifetime: 1.0,
                            damage: 1,
                            color: BOULDER_COLOR,
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
                    Faction::Enemy
                ))
                .id();

                spawn_event_writer.write(event::SpawnEvent { entity: enemy });
        }
    }
}

/// System to move boulder enemies toward player with rolling motion
pub fn move_boulders(
    mut boulder_query: Query<(&mut Transform, &Boulder)>,
    player_query: Query<&Transform, (With<player::Player>, Without<Boulder>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut boulder_transform, boulder) in &mut boulder_query {
            // Calculate direction to player
            let direction =
                (player_transform.translation - boulder_transform.translation).normalize_or_zero();
            let horizontal_direction = Vec3::new(direction.x, 0.0, direction.z).normalize_or_zero();

            // Calculate movement for this frame
            let movement_distance = boulder.speed * time.delta_secs();
            let movement_vector = horizontal_direction * movement_distance;

            // Move enemy toward player
            boulder_transform.translation += movement_vector;

            boulder_transform.translation.x += boulder.jitter * (fastrand::f32() - 0.5); // Add slight random jitter
            boulder_transform.translation.z += boulder.jitter * (fastrand::f32() - 0.5); // Add slight random jitter

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
                boulder_transform.rotation =
                    boulder_transform.rotation * roll_rotation * pitch_rotation;
            }
        }
    }
}


pub fn boulder_combat_ai(
    mut boulder_query: Query<(Entity, &Transform, &AbilitySlot<Slam>, &AbilitySlot<Shotgun>, &AbilitySlot<Dash>, &Living, &mut Boulder)>,
    player_query: Query<&Transform, (With<player::Player>, Without<Boulder>)>,
    mut slam_action: EventWriter<AttemptCastEvent<Slam>>,
    mut shotgun_action: EventWriter<AttemptCastEvent<Shotgun>>,
    mut dash_action: EventWriter<AttemptCastEvent<Dash>>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (boulder_entity, boulder_transform, slam_ability, _shotgun_ability, dash_ability, living, mut boulder) in &mut boulder_query {
            let distance = boulder_transform.translation.distance(player_transform.translation);

            let enraged = living.health_fraction() < 0.5;

            if enraged {
                boulder.speed = 15.0;
                boulder.jitter = 1.0;
            }
            
            // Use dash to close distance if far away (aggressive pursuit)
            if distance > 8.0 && enraged  {
                let direction = (player_transform.translation - boulder_transform.translation)
                    .normalize_or_zero()
                    .with_y(0.0);
                    
                dash_action.write(AttemptCastEvent {
                    caster: boulder_entity,
                    params: DashParams::Directional(direction),
                    _marker: std::marker::PhantomData::default(),
                });
            }

            // Use slam if in range
            if distance <= slam_ability.ability.range {
                slam_action.write(AttemptCastEvent {
                    caster: boulder_entity,
                    params: SlamParams {
                        target_position: player_transform.translation,
                    },
                    _marker: std::marker::PhantomData::default(),
                });
            }

            // Use shotgun if out of slam range but within shooting range
            if distance >= 10.0 { // Shooting range
                shotgun_action.write(AttemptCastEvent {
                    caster: boulder_entity,
                    params: ShotgunParams {
                        target_position: player_transform.translation,
                    },
                    _marker: std::marker::PhantomData::default(),
                });
            }
        }
    }
}

impl Enemy for Boulder {
    type SpawnParams = BoulderSpawnParams;

    fn add_systems(app: &mut App) {
        app.add_systems(Update, (
            handle_boulder_spawn,
            move_boulders,
            boulder_combat_ai,
        ));
    }
}