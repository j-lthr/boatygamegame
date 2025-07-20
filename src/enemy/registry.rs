use std::sync::Arc;
use bevy::prelude::*;
use crate::common::BundleInjector;
use crate::enemy::spawn::SpawnInfo;

#[derive(Clone, Copy)]
pub struct EnemyID(&'static str);

impl EnemyID {
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

pub struct Enemy {
    id: EnemyID,
    spawn_info: SpawnInfo,
    components: Arc<dyn BundleInjector + Send + Sync>,
}

impl Enemy {
    pub fn id(&self) -> EnemyID {
        self.id
    }

    pub fn spawn_info(&self) -> &SpawnInfo {
        &self.spawn_info
    }

    pub fn add_to_entity(&self, entity: &mut EntityCommands) {
        self.components.add_to_entity(entity);
    }
}

#[derive(Resource, Default)]
pub struct EnemyRegistry {
    enemies: Vec<Enemy>,
}

impl EnemyRegistry {
    pub fn register_enemy(&mut self, enemy: Enemy) {
        self.enemies.push(enemy);
    }

    pub fn enemies(&self) -> impl Iterator<Item=&Enemy> {
        self.enemies.iter()
    }

}

pub fn register_enemies(mut registry: ResMut<EnemyRegistry>) {

}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, register_enemies);
}