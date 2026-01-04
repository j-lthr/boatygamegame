use crate::common::{BundleInjector, EntityModifier};
use bevy::prelude::*;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct EnemyID(&'static str);

impl EnemyID {
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct EnemyConfig {
    pub min_level: i32,
    pub max_level: Option<i32>,
    pub num_slots: i32,
}

impl Default for EnemyConfig {
    fn default() -> Self {
        Self {
            min_level: 0,
            max_level: None,
            num_slots: 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Enemy {
    id: EnemyID,
    config: EnemyConfig,
    components: Arc<dyn EntityModifier + Send + Sync>,
}

impl Enemy {
    pub fn id(&self) -> EnemyID {
        self.id
    }

    pub fn add_to_entity(&self, entity: &mut EntityCommands) {
        self.components.add_to_entity(entity);
    }

    pub fn from_components(
        id: &'static str,
        bundle: impl Bundle + Clone + std::fmt::Debug,
    ) -> Self {
        Self {
            id: EnemyID(id),
            config: Default::default(),
            components: Arc::new(BundleInjector(bundle)),
        }
    }

    pub fn with_min_level(mut self, level: i32) -> Self {
        self.config.min_level = level;
        self
    }

    pub fn with_num_slots(mut self, num_slots: i32) -> Self {
        self.config.num_slots = num_slots;
        self
    }

    pub fn config(&self) -> EnemyConfig {
        self.config
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

    pub fn enemies(&self) -> impl ExactSizeIterator<Item = &Enemy> {
        self.enemies.iter()
    }

    pub fn get(&self, id: &str) -> Option<&Enemy> {
        self.enemies.iter().find(|enemy| enemy.id.as_str() == id)
    }
}
