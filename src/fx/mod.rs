pub mod blood;
pub mod fm;
pub mod stars;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    stars::plugin(app);
}
