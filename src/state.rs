
use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
}

#[derive(Resource)]
pub struct GameScore {
    pub current: i32,
    pub kills: i32,
    pub combo: i32,
    pub combo_timer: Timer,
}


impl Default for GameScore {
    fn default() -> Self {
        Self {
            current: 0,
            kills: 0,
            combo: 0,
            combo_timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}
