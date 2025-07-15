use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Copy, Clone)]
pub struct Modifier {
    multiplicative_factor: f32,
    additive_factor: f32,
}

impl Default for Modifier {
    fn default() -> Self {
        Self {
            multiplicative_factor: 1.0,
            additive_factor: 0.0,
        }
    }
}

impl Modifier {
    pub fn apply_to_base_value(&self, base_value: f32) -> f32 {
        (1.0 + self.additive_factor) * self.multiplicative_factor * base_value
    }

    pub fn modify_multiplicative(&mut self, value: f32) {
        self.multiplicative_factor *= value;
    }

    pub fn modify_additive(&mut self, value: f32) {
        self.additive_factor += value;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ModifierID(&'static str);

impl ModifierID {
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub enum StatKind {
    Additive(f32),
    Multiplicative(f32),
}

#[derive(Copy, Clone, Debug)]
pub struct Stat {
    pub id: ModifierID,
    pub kind: StatKind,
}

impl Stat {
    pub const fn additive(id: ModifierID, value: f32) -> Self {
        Self {
            id,
            kind: StatKind::Additive(value),
        }
    }

    pub const fn multiplicative(id: ModifierID, value: f32) -> Self {
        Self {
            id,
            kind: StatKind::Multiplicative(value),
        }
    }
}

#[derive(Component, Default)]
pub struct ModifierStack {
    stack: HashMap<ModifierID, Modifier>,
}

impl ModifierStack {
  
    pub fn get(&self, id: ModifierID) -> Option<Modifier> {
        self.stack.get(&id).cloned()
    }

    pub fn add_stat(&mut self, stat: Stat) {
        if !self.stack.contains_key(&stat.id) {
            self.stack.insert(stat.id, Default::default());
        }

        let modifier = self.stack.get_mut(&stat.id).unwrap();
        match stat.kind {
            StatKind::Additive(value) => modifier.modify_additive(value),
            StatKind::Multiplicative(value) => modifier.modify_multiplicative(value),
        }
    }

    pub fn add_multiplicative_modifier(&mut self, id: ModifierID, value: f32) {
        self.add_stat(Stat { id, kind: StatKind::Multiplicative(value) });
    }

    pub fn add_additive_modifier(&mut self, id: ModifierID, value: f32) {
        self.add_stat(Stat { id, kind: StatKind::Additive(value) });
    }
}


pub fn apply_modifier_if_present(query_result: Option<&ModifierStack>, id: ModifierID, base_value: f32) -> f32 {
    let modifier = query_result.and_then(|stack| stack.get(id)).unwrap_or_default();

    modifier.apply_to_base_value(base_value)
}

pub const DAMAGE_MODIFIER: ModifierID = ModifierID("basic-damage");

pub const PLAYER_SPEED_MODIFIER: ModifierID = ModifierID("player-speed");
pub const PROJECTILE_SPEED_MODIFIER: ModifierID = ModifierID("projectile-speed");
pub const PROJECTILE_COUNT_MODIFIER: ModifierID = ModifierID("projectile-count");
pub const PROJECTILE_DURATION_MODIFIER: ModifierID = ModifierID("projectile-duration");
pub const AOE_RADIUS_MODIFIER: ModifierID = ModifierID("aoe-radius");