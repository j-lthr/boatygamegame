use bevy::prelude::*;

#[derive(Component)]
pub struct Living {
    pub health: i32,
    pub max_health: i32,
}
