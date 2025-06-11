use bevy::prelude::*;
use std::f32;

use super::{AttemptSpawnEvent, Enemy};
use crate::ability::missile_launcher::MissileLauncher;
use crate::ability::missile_launcher::MissileParams;
use crate::ability::{AbilitySlot, AttemptCastEvent};
use crate::common::Faction;
use crate::common::Living;
use crate::common::{self, Inertia};
use crate::enemy::common::FirstOrderMovement;
use crate::enemy::common::FollowMovementMode;
use crate::enemy::common::FollowTarget;
use crate::event;
use crate::init::DespawnOnReset;
use crate::loot::DropTableBuilder;
use crate::player;
use crate::player::Player;
use crate::rune::*;
use crate::utils::normal_dist_1d;

pub const SNIPER_COLOR: Color = Color::srgb(5.0, 0.0, 0.0);

// Component for boulder enemies
#[derive(Component, Clone)]
pub struct Sniper;

#[derive(Clone)]
pub struct SniperSpawnParams {
    pub position: Vec3,
    pub pack_size: i32,
}

/// System to handle boulder spawn events
pub fn handle_sniper_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_attempts: EventReader<AttemptSpawnEvent<Sniper>>,
    mut spawn_event_writer: EventWriter<event::SpawnEvent>,
    player_query: Query<Entity, With<Player>>,
) {
    for spawn_attempt in spawn_attempts.read() {
        let mesh = meshes.add(Sphere::new(0.8));

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
                            emissive: SNIPER_COLOR.into(), // Slightly glowing
                            ..default()
                        })),
                        Transform::from_translation(spawn_position),
                        Sniper,
                        FollowTarget {
                            target: player,
                            mode: FollowMovementMode::Ranged {
                                preferred_distance: 30.0,
                                rotation_speed: 0.1,
                            },
                        },
                        FirstOrderMovement {
                            speed: 50.0,
                            jitter: 0.0,
                        },
                        AbilitySlot {
                            cooldown: Timer::from_seconds(
                                normal_dist_1d(2.0, 0.01).abs(),
                                TimerMode::Once,
                            ),
                            name: "Missile",
                            ability: MissileLauncher {
                                missile_count: 1,
                                spread: 0.45,
                                speed: 50.0,
                                lifetime: 2.0,
                                damage: 100,
                                explosion_radius: 5.0,
                                color: Color::linear_rgb(50.0, 0.0, 0.0),
                                tracking_strength: 5.0,
                                lock_distance: 15.0,
                            },
                        },
                        common::Living {
                            health: 8,
                            max_health: 8,
                        },
                        Faction::Enemy,
                        DropTableBuilder::new()
                            .add_rune(0.0, SPEED_RUNE)
                            .add_rune(2.0, HEAL_RUNE)
                            .add_rune(0.5, MULTISHOT_RUNE)
                            .build(),
                        Inertia {
                            prev_pos: spawn_position,
                            damping: 0.0,
                        },
                        DespawnOnReset,
                    ))
                    .id();

                spawn_event_writer.write(event::SpawnEvent { entity: enemy });
            }
        }
    }
}

pub fn sniper_combat_ai(
    mut sniper_query: Query<
        (Entity, &Transform, &AbilitySlot<MissileLauncher>, &Living),
        With<Sniper>,
    >,
    player_query: Query<(Entity, &Transform), (With<player::Player>, Without<Sniper>)>,
    mut missile_action: EventWriter<AttemptCastEvent<MissileLauncher>>,
) {
    if let Ok((player_entity, player_transform)) = player_query.single() {
        for (boulder_entity, boulder_transform, missile_ability, movement) in &mut sniper_query {
            let distance = boulder_transform
                .translation
                .distance(player_transform.translation);

            missile_action.write(AttemptCastEvent {
                caster: boulder_entity,
                params: MissileParams {
                    target_entity: player_entity,
                },
                _marker: std::marker::PhantomData::default(),
            });
        }
    }
}

pub fn sniper_cooldown_visual(
    mut sniper_query: Query<(&mut Transform, &AbilitySlot<MissileLauncher>), With<Sniper>>,
    time: Res<Time>,
) {
    for (mut sniper_transform, missile_ability) in &mut sniper_query {
        sniper_transform.scale = sniper_transform.scale.lerp(
            Vec3::splat(missile_ability.cooldown.fraction_remaining() + 0.2),
            time.delta_secs() * 10.0,
        );
    }
}

impl Enemy for Sniper {
    type SpawnParams = SniperSpawnParams;

    fn add_systems(app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_sniper_spawn,
                sniper_combat_ai,
                sniper_cooldown_visual,
            ),
        );
    }
}
