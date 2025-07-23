use super::components::*;
use crate::ability::DynamicAbility;
use crate::ability::components::common::Lifetime;
use crate::ability::components::projectile::{
    DamageOnCollision, DespawnOnCollision, LinearMovement, SimpleCollider,
};
use crate::ability::components::spawn::RadialSubCastOffset;
use crate::ability::components::subcast::{CastOnDespawn, SubCastOnce};
use crate::common::{BundleInjector, BundleWrapper, HealthBundle};
use crate::enemy::spawn::SpawnInfo;
use crate::loot::DropTableBuilder;
use avian3d::parry::partitioning::SimdBestFirstVisitor;
use bevy::prelude::*;
use crate::rune::*;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct EnemyID(&'static str);

impl EnemyID {
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

#[derive(Copy, Clone)]
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

pub struct Enemy {
    id: EnemyID,
    config: EnemyConfig,
    components: Arc<dyn BundleInjector + Send + Sync>,
}

impl Enemy {
    pub fn id(&self) -> EnemyID {
        self.id
    }

    pub fn add_to_entity(&self, entity: &mut EntityCommands) {
        self.components.add_to_entity(entity);
    }

    pub fn from_components(id: &'static str, bundle: impl Bundle + Clone + Send) -> Self {
        Self {
            id: EnemyID(id),
            config: Default::default(),
            components: Arc::new(BundleWrapper(bundle)),
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
}
