use std::f32;

use bevy::prelude::*;

use crate::{enemy::{boulder::{Boulder, BoulderSpawnParams}, sniper::{Sniper, SniperSpawnParams}, AttemptSpawnEvent}, utils::normal_dist_1d};


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
            wave_timer: Timer::from_seconds(4.0, TimerMode::Repeating),
            wave_index: 0,
        }
    }
}

pub fn spawn(mut commands: Commands, mut state: ResMut<SpawnerState>, time: ResMut<Time>, mut target_query: Query<&Transform, With<SpawnerTarget>>) {
    state.wave_timer.tick(time.delta());

    let num_packs = 1 + state.wave_index / 8;
    

    if state.wave_timer.finished() {
        state.wave_timer.reset();
        
        for target_transform in target_query {
            for _ in 0..num_packs {
                let spawn_radius = normal_dist_1d(20.0, 2.0).abs();
                let spawn_angle = fastrand::f32() * f32::consts::PI;

                let spawn_pos = Vec2::from_angle(spawn_angle) * spawn_radius;


                if state.wave_index % 2 == 0 {
                    commands.send_event(
                        AttemptSpawnEvent::<Boulder>::new(
                            BoulderSpawnParams {
                                position: target_transform.translation + spawn_pos.xxy().with_y(0.0),
                                pack_size: 3,
                            }
                        )
                    );
                } else {
                    commands.send_event(
                        AttemptSpawnEvent::<Sniper>::new(
                            SniperSpawnParams {
                                position: target_transform.translation + spawn_pos.xxy().with_y(0.0),
                                pack_size: 1,
                            }
                        )
                    );
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