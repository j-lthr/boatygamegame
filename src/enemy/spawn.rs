use std::{f32, time::Duration};

use bevy::prelude::*;

use crate::init::DespawnOnReset;
use crate::{
    common::Faction, event::SpawnEvent, utils::normal_dist_1d
};
use crate::enemy::registry::EnemyRegistry;

#[derive(Resource)]
pub struct SpawnerState {
    pub wave_timer: Timer,
    pub wave_index: i32,
}

#[derive(Component)]
pub struct SpawnerTarget;

impl FromWorld for SpawnerState {
    fn from_world(_world: &mut World) -> Self {
        SpawnerState {
            wave_timer: Timer::from_seconds(6.0, TimerMode::Repeating),
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
    mut spawn_events: EventWriter<SpawnEvent>,
) {
    state.wave_timer.tick(time.delta());

    let enemy = fastrand::choice(enemy_registry.enemies()).unwrap();
    
    if state.wave_timer.finished() {
        
        if state.wave_index < 3 || state.wave_index == 32  {
            let mut current_duration = state.wave_timer.duration().as_secs();
            current_duration -= 1;
            state.wave_timer.set_duration(Duration::from_secs(current_duration));
        }

        for (target, target_transform) in target_query {
            let spawn_pos = random_spawn_pos(target_transform.translation, 30.0, 10.0);

            let mut entity = commands.spawn((
                Transform::from_translation(spawn_pos),
                Faction::Enemy,
                SpawnInfo {
                    target
                },
                DespawnOnReset
            ));

            enemy.add_to_entity(&mut entity);

            spawn_events.write(SpawnEvent {
                entity: entity.id()
            });
        }

        state.wave_timer.reset();
        state.wave_index += 1;
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<SpawnerState>();
    app.init_resource::<EnemyRegistry>();
    app.add_systems(Update, spawn);
}
