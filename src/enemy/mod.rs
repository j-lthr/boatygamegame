use std::marker::PhantomData;
use bevy::prelude::*;

pub mod boulder;
pub mod spawn;
pub mod common;
pub mod sniper;

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
        Self { params, _marker: PhantomData }
    }
}

fn register_enemy<T: Enemy>(app: &mut App) {
    app.add_event::<AttemptSpawnEvent<T>>();
    T::add_systems(app);
}

pub fn plugin(app: &mut App) {
    app.add_plugins(
        (
            register_enemy::<boulder::Boulder>,
            register_enemy::<sniper::Sniper>,
            spawn::plugin
        )
    );

    common::register(app);
}