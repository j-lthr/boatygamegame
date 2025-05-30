use bevy::prelude::*;

#[derive(Event)]
pub struct SpawnEvent {
    pub entity: Entity,
}