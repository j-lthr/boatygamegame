use bevy::ecs::spawn;
use bevy::prelude::*;
use std::f32;

use super::{AttemptSpawnEvent, Enemy};
use crate::ability::dash::{Dash, DashParams};
use crate::ability::missile_launcher::MissileLauncher;
use crate::ability::missile_launcher::MissileParams;
use crate::ability::basic_projectile_attack::{BasicProjectileAttack, BasicProjectileAttackParams};
use crate::ability::slam::{Slam, SlamParams};
use crate::ability::{AbilitySlot, AttemptCastEvent};
use crate::common::{self, Inertia};
use crate::common::Faction;
use crate::common::Living;
use crate::enemy::common::FirstOrderMovement;
use crate::enemy::common::FollowTarget;
use crate::enemy::common::FollowMovementMode;
use crate::enemy::common::MoveEvent;
use crate::event;
use crate::init::DespawnOnReset;
use crate::loot::DropTableBuilder;
use crate::player;
use crate::player::Player;
use crate::procedural;
use crate::rune::*;
use crate::utils::normal_dist_1d;

pub const BOULDER_COLOR: Color = Color::srgb(4.0, 2.0, 4.0);

// Component for boulder enemies
#[derive(Component, Clone)]
pub struct Boulder;

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
    player_query: Query<Entity, With<Player>>,
) {
    for spawn_attempt in spawn_attempts.read() {
        let mesh = meshes.add(procedural::rock::generate_rock_mesh(
            &procedural::rock::RockConfig::default(),
        ));

        if let Ok(player) = player_query.single() {
            for i in 0..spawn_attempt.params.pack_size {
                let offset_angle =
                    i as f32 / spawn_attempt.params.pack_size as f32 * f32::consts::PI * 2.0;

                let spawn_position = spawn_attempt.params.position
                                + vec3(f32::cos(offset_angle), 0.0, f32::sin(offset_angle));

                let enemy = commands
                    .spawn((
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(10.0, 5.0, 10.0),
                            emissive: BOULDER_COLOR.into(), // Slightly glowing
                            ..default()
                        })),
                        Transform::from_translation(
                            spawn_position,
                        ),
                        Boulder,
                        FollowTarget {
                            target: player,
                            mode: FollowMovementMode::ToMeleeRange,
                        },
                        FirstOrderMovement {
                            speed: 10.0,
                            jitter: 0.1,
                        },
                        AbilitySlot {
                            cooldown: Timer::from_seconds(0.25, TimerMode::Once),
                            name: "Slam",
                            ability: Slam {
                                range: normal_dist_1d(3.0, 0.5),
                                damage: 33,
                                knockback_force: 3.0,
                            },
                        },
                        AbilitySlot {
                            cooldown: Timer::from_seconds(2.0, TimerMode::Once),
                            name: "Shotgun",
                            ability: BasicProjectileAttack {
                                bullet_count: 5,
                                spread: 0.05,
                                speed: 50.0,
                                lifetime: 2.0,
                                damage: 5,
                                color: BOULDER_COLOR,
                            },
                        },
                        AbilitySlot {
                            cooldown: Timer::from_seconds(10.0, TimerMode::Once),
                            name: "Missile",
                            ability: MissileLauncher {
                                missile_count: 1,
                                spread: 0.45,
                                speed: 30.0,
                                lifetime: 10.0,
                                damage: 20,
                                explosion_radius: 1.0,
                                color: Color::linear_rgb(50.0, 0.0, 0.0),
                                tracking_strength: 10.0,
                                lock_distance: 10.0,
                            },
                        },
                        AbilitySlot {
                            cooldown: Timer::from_seconds(3.0, TimerMode::Once),
                            name: "Dash",
                            ability: Dash { range: 6.0 },
                        },
                        common::Living {
                            health: 32,
                            max_health: 32,
                        },
                        Faction::Enemy,
                        DropTableBuilder::new()
                            .add_rune(1.0, SPEED_RUNE)
                            .add_rune(2.0, HEAL_RUNE)
                            .build(),
                        Inertia {
                            prev_pos: spawn_position,
                            damping: 0.0,
                        },
                        DespawnOnReset
                    ))
                    .id();

                spawn_event_writer.write(event::SpawnEvent { entity: enemy });
            }
        }
    }
}


/// System to move boulder enemies toward player with rolling motion
pub fn boulder_rotation_effect(
    mut boulder_query: Query<(&mut Transform, &Inertia), With<Boulder>>,
    time: Res<Time>,
) {
    for (mut boulder_transform, boulder_inertia) in &mut boulder_query {

        let delta = boulder_transform.translation - boulder_inertia.prev_pos;

        // Add rolling motion
        if delta.length() > 0.0 {
            // Assume boulder radius for rolling calculation (adjust as needed)
            let boulder_radius = 0.5; // Adjust this based on your boulder size

            // Calculate rotation angles based on movement
            let roll_angle = delta.x / boulder_radius; // Roll around Z-axis for X movement
            let pitch_angle = -delta.z / boulder_radius; // Pitch around X-axis for Z movement (negative for correct direction)

            // Apply rolling rotation
            let roll_rotation = Quat::from_rotation_z(roll_angle);
            let pitch_rotation = Quat::from_rotation_x(pitch_angle);

            // Combine rotations and apply to current rotation
            boulder_transform.rotation =
                boulder_transform.rotation * roll_rotation * pitch_rotation;
        }
    }
}

pub fn boulder_combat_ai(
    mut boulder_query: Query<
        (
            Entity,
            &Transform,
            &AbilitySlot<Slam>,
            &AbilitySlot<BasicProjectileAttack>,
            &AbilitySlot<Dash>,
            &Living,
            &mut FirstOrderMovement,
        ),
        With<Boulder>,
    >,
    player_query: Query<(Entity, &Transform), (With<player::Player>, Without<Boulder>)>,
    mut slam_action: EventWriter<AttemptCastEvent<Slam>>,
    mut shotgun_action: EventWriter<AttemptCastEvent<BasicProjectileAttack>>,
    mut dash_action: EventWriter<AttemptCastEvent<Dash>>,
    mut missile_action: EventWriter<AttemptCastEvent<MissileLauncher>>,
) {
    if let Ok((player_entity, player_transform)) = player_query.single() {
        for (
            boulder_entity,
            boulder_transform,
            slam_ability,
            _shotgun_ability,
            dash_ability,
            living,
            mut movement,
        ) in &mut boulder_query
        {
            let distance = boulder_transform
                .translation
                .distance(player_transform.translation);

            let enraged = living.health_fraction() < 0.5;

            if enraged {
                movement.speed = 15.0;
                movement.jitter = 0.2;
            }

            // Use dash to close distance if far away (aggressive pursuit)
            if distance > 8.0 && enraged {
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

            /*// Use shotgun if out of slam range but within shooting range
            if distance >= 10.0 { // Shooting range
                shotgun_action.write(AttemptCastEvent {
                    caster: boulder_entity,
                    params: ShotgunParams {
                        target_position: player_transform.translation,
                    },
                    _marker: std::marker::PhantomData::default(),
                });

                missile_action.write(AttemptCastEvent {
                    caster: boulder_entity,
                    params: MissileParams {
                        target_entity: player_entity
                    },
                    _marker: std::marker::PhantomData::default(),
                });
            }*/
        }
    }
}

impl Enemy for Boulder {
    type SpawnParams = BoulderSpawnParams;

    fn add_systems(app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_boulder_spawn,
                boulder_combat_ai,
                boulder_rotation_effect
            ),
        );
    }
}
