use super::*;

pub mod blast;
pub mod common;
pub mod dash;
pub mod events;
pub mod projectile;
pub mod spawn;
pub mod subcast;
pub mod visual;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        projectile::plugin,
        subcast::plugin,
        spawn::plugin,
        common::plugin,
        blast::plugin,
        events::plugin,
        visual::plugin,
        dash::plugin,
    ));
}
