use bevy::prelude::*;
use std::marker::PhantomData;

pub mod components;
pub mod spawn;
pub mod registry;

pub mod enemies;

pub trait Enemy: Component + Clone + Send + Sync + 'static {
    type SpawnParams: Clone + Send + Sync + 'static;

    fn add_systems(app: &mut App);
}

#[derive(Event)]
pub struct AttemptSpawnEvent<T: Enemy> {
    pub params: T::SpawnParams,
    pub _marker: PhantomData<T>,
}
impl<T: Enemy> AttemptSpawnEvent<T> {
    fn new(params: T::SpawnParams) -> Self {
        Self {
            params,
            _marker: PhantomData,
        }
    }
}

fn register_enemy<T: Enemy>(app: &mut App) {
    app.add_event::<AttemptSpawnEvent<T>>();
    T::add_systems(app);
}

pub fn plugin(app: &mut App) {
    spawn::plugin(app);
    components::plugin(app);
    enemies::plugin(app);
}
