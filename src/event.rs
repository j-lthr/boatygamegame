use bevy::prelude::*;

#[derive(Event)]
pub struct SpawnEvent {
    pub entity: Entity,
}

#[derive(Event)]
pub struct DamageEvent {
    pub target: Entity,
    pub source: Option<Entity>, // Optional source entity for tracking who caused damage
    pub damage: i32,
    pub position: Vec3, // World position where damage occurred for UI
}