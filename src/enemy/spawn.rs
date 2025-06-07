use std::f32;

use bevy::prelude::*;

use crate::{enemy::{AttemptSpawnEvent, Boulder, BoulderSpawnParams}, utils::normal_dist_1d};


#[derive(Resource)]
pub struct SpawnerState {
    wave_timer: Timer,
}

#[derive(Component)]
pub struct SpawnerTarget;

impl FromWorld for SpawnerState {
    fn from_world(world: &mut World) -> Self {
        SpawnerState {
            wave_timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}



pub fn spawn(mut state: ResMut<SpawnerState>, time: ResMut<Time>, mut target_query: Query<&Transform, With<SpawnerTarget>>, mut boulder_spawns: EventWriter<AttemptSpawnEvent<Boulder>>) {
    state.wave_timer.tick(time.delta());

    if state.wave_timer.finished() {
        state.wave_timer.reset();
        
        for target_transform in target_query {
            for _ in 0..3 {
                let spawn_radius = normal_dist_1d(20.0, 2.0).abs();
                let spawn_angle = fastrand::f32() * f32::consts::PI;

                let spawn_pos = Vec2::from_angle(spawn_angle) * spawn_radius;

                boulder_spawns.write(
                    AttemptSpawnEvent::new(
                        BoulderSpawnParams {
                            position: target_transform.translation + spawn_pos.xxy().with_y(0.0),
                            pack_size: 3,
                        }
                    )
                );
            }
        }
    }   
}


pub fn register(app: &mut App) {
    app.init_resource::<SpawnerState>();
    app.add_systems(Update, spawn);
}