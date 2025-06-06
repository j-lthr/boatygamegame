use std::marker::PhantomData;
use bevy::prelude::*;

pub mod boulder;

pub trait Enemy: Component + Clone + Send + Sync + 'static {
    type SpawnParams: Clone + Send + Sync + 'static;

    fn add_systems(app: &mut App);
}

#[derive(Event)]
pub struct AttemptSpawnEvent<T: Enemy> {
    pub params: T::SpawnParams,
    pub _marker: PhantomData<T>,
}

pub struct EnemyPlugin<T: Enemy> {
    _marker: PhantomData<T>,
}

impl<T: Enemy> EnemyPlugin<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: Enemy> Plugin for EnemyPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_event::<AttemptSpawnEvent<T>>();
        T::add_systems(app);
    }
}

// Re-export for backward compatibility
pub use boulder::{Boulder, BoulderSpawnParams};
pub use boulder::{move_boulders as move_enemies, boulder_combat_ai as enemy_combat_ai};