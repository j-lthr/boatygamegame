use std::{f32, time::Duration};

use bevy::prelude::*;

use crate::init::DespawnOnReset;
use crate::{
    common::Faction, event::SpawnEvent, utils::normal_dist_1d
};

#[derive(Event)]
pub struct SpawnEnemyEvent {
    pub enemy_id: String,
    pub position: Vec3,
    pub target: Entity,
}

use crate::enemy::registry::*;
use crate::loot::DropScale;

#[derive(Resource)]
pub struct SpawnerState {
    pub wave_timer: Timer,
    pub wave_index: i32,
}

#[derive(Component)]
pub struct SpawnerTarget;

impl FromWorld for SpawnerState {
    fn from_world(_world: &mut World) -> Self {
        let mut timer = Timer::from_seconds(10.0, TimerMode::Repeating);

        SpawnerState {
            wave_timer: timer,
            wave_index: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub enum SpawnRequirement {
    MinWave(i32),
}

#[derive(Clone, Copy, Component)]
pub struct SpawnInfo {
    pub target: Entity,
}


pub fn random_spawn_pos(center: Vec3, radius_avg: f32, radius_std: f32) -> Vec3 {
    let spawn_radius = normal_dist_1d(radius_avg, radius_std).abs();
    let spawn_angle = 2.0 * fastrand::f32() * f32::consts::PI;

    center
        + (Vec2::from_angle(spawn_angle) * spawn_radius)
            .xxy()
            .with_y(0.0)
}

pub fn spawn(
    mut commands: Commands,
    mut state: ResMut<SpawnerState>,
    enemy_registry: Res<EnemyRegistry>,
    time: Res<Time>,
    target_query: Query<(Entity, &Transform), With<SpawnerTarget>>,
    enemy_query: Query<(), With<SpawnInfo>>,
    mut spawn_events: EventWriter<SpawnEvent>,
) {

    let time_scale = if enemy_query.iter().len() == 0 {
        10.0
    } else {
        1.0
    };

    state.wave_timer.tick(time.delta().mul_f32(time_scale));

    let level = state.wave_index + 1;
    let mut enemy_slots = level;

    
    if state.wave_timer.just_finished() {
        
        // if state.wave_index < 3 || state.wave_index == 32  {
        //     let mut current_duration = state.wave_timer.duration().as_secs();
        //     current_duration -= 1;
        //     state.wave_timer.set_duration(Duration::from_secs(current_duration));
        // }



        let mut eligible_enemies: Vec<_> = enemy_registry.enemies().collect();

        while enemy_slots > 0 {

            eligible_enemies.retain(|enemy| {
                let config = enemy.config();

                level >= config.min_level && enemy_slots >= config.num_slots
            });



            let enemy = fastrand::choice(eligible_enemies.iter()).unwrap();

            enemy_slots -= enemy.config().num_slots;

            for (target, target_transform) in target_query {
                let spawn_pos = random_spawn_pos(target_transform.translation, 30.0, 0.0);

                let mut entity = commands.spawn((
                    Transform::from_translation(spawn_pos),
                    Faction::Enemy,
                    SpawnInfo {
                        target,
                    },
                    DropScale {
                        scale: enemy.config().num_slots
                    },
                    DespawnOnReset
                ));

                enemy.add_to_entity(&mut entity);

                spawn_events.write(SpawnEvent {
                    entity: entity.id()
                });
            }

        }

        state.wave_index += 1;
        state.wave_timer.reset();
    }
}

pub fn handle_spawn_enemy_events(
    mut commands: Commands,
    mut spawn_events: EventReader<SpawnEnemyEvent>,
    enemy_registry: Res<EnemyRegistry>,
    mut game_spawn_events: EventWriter<SpawnEvent>,
) {
    for event in spawn_events.read() {
        if let Some(enemy) = enemy_registry.get(&event.enemy_id) {
            let mut entity = commands.spawn((
                Transform::from_translation(event.position),
                Faction::Enemy,
                SpawnInfo {
                    target: event.target,
                },
                DropScale {
                    scale: enemy.config().num_slots
                },
                DespawnOnReset
            ));

            enemy.add_to_entity(&mut entity);

            game_spawn_events.write(SpawnEvent {
                entity: entity.id()
            });
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<SpawnerState>();
    app.init_resource::<EnemyRegistry>();
    app.add_event::<SpawnEnemyEvent>();
    app.add_systems(Update, (spawn, handle_spawn_enemy_events));
}
