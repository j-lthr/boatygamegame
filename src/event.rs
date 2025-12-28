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
    pub impact_velocity: Option<Vec3>,
}

#[derive(Event)]
pub struct DeathEvent {
    pub entity: Entity,
}

pub fn plugin(app: &mut App) {
    app.add_event::<SpawnEvent>();
    app.add_event::<DamageEvent>();
    app.add_event::<DeathEvent>();
}

pub trait EventSource<Input>: Send + Sync + std::fmt::Debug {
    fn send_event(&self, commands: Commands, input: Input);
}
