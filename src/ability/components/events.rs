pub use bevy::prelude::*;

#[derive(Event)]
pub struct OnActiveDespawn;

#[derive(Event)]
pub struct OnCollision {
    pub target: Entity,
}

pub fn plugin(app: &mut App) {
    app.add_event::<OnActiveDespawn>();
    app.add_event::<OnCollision>();
}