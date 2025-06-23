use super::*;
use bevy::prelude::*;

pub mod projectile;
pub mod subcast;
pub mod spawn;
pub mod common;
pub mod blast;

pub fn plugin(app: &mut App) {
    app.add_plugins(
        (
            projectile::plugin,
            subcast::plugin,
            spawn::plugin,
            common::plugin,
            blast::plugin,
        )
    );
}