use bevy::prelude::*;

pub mod components;
pub mod registry;
pub mod spawn;

pub mod enemies;

pub fn plugin(app: &mut App) {
    spawn::plugin(app);
    components::plugin(app);
    enemies::plugin(app);
}
