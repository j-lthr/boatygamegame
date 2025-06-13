use std::{f32, time::Duration};

use bevy::prelude::*;

use crate::{
    enemy::{
        AttemptSpawnEvent,
        boulder::{Boulder, BoulderSpawnParams},
        sniper::{Sniper, SniperSpawnParams},
    },
    utils::normal_dist_1d,
};

#[derive(Resource)]
pub struct SpawnerState {
    pub wave_interval: f32,
    pub wave_timer: Timer,
    pub wave_index: i32,
}

#[derive(Component)]
pub struct SpawnerTarget;

impl FromWorld for SpawnerState {
    fn from_world(_world: &mut World) -> Self {
        SpawnerState {
            wave_interval: 6.0,
            wave_timer: Timer::from_seconds(6.0, TimerMode::Repeating),
            wave_index: 0,
        }
    }
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
    time: ResMut<Time>,
    target_query: Query<&Transform, With<SpawnerTarget>>,
) {
    state.wave_timer.tick(time.delta());

    let num_packs = 1 + state.wave_index / 8;

    

    if state.wave_timer.finished() {
        
        if state.wave_index < 3 || state.wave_index == 32  {
            let mut current_duration = state.wave_timer.duration().as_secs();
            current_duration -= 1;
            state.wave_timer.set_duration(Duration::from_secs(current_duration));
        }

        if state.wave_index > 32 && state.wave_index % 16 == 0 {
            let mut current_duration = state.wave_timer.duration().as_secs_f32();
            current_duration *= 0.8;
            current_duration = current_duration.max(1.0);
            state.wave_timer.set_duration(Duration::from_secs_f32(current_duration));
        }

        state.wave_timer.reset();

        for target_transform in target_query {
            for _ in 0..num_packs {
                if state.wave_index % 2 == 0 {
                    commands.send_event(AttemptSpawnEvent::<Boulder>::new(BoulderSpawnParams {
                        position: random_spawn_pos(target_transform.translation, 20.0, 2.0),
                        pack_size: 3 * (state.wave_index / 16 + 1),
                    }));
                } else {
                    commands.send_event(AttemptSpawnEvent::<Sniper>::new(SniperSpawnParams {
                        position: random_spawn_pos(target_transform.translation, 50.0, 10.0),
                        pack_size: 1 * (state.wave_index / 16 + 1),
                    }));
                }
            }
        }

        state.wave_index += 1;
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<SpawnerState>();
    app.add_systems(Update, spawn);
}
