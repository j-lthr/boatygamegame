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

#[derive(Component, Default)]
pub struct ModifierStack {
    stack: HashMap<ModifierID, Modifier>,
}

impl ModifierStack {
  
    pub fn get(&self, id: ModifierID) -> Option<Modifier> {
        self.stack.get(&id).cloned()
    }
    pub fn add_multiplicative_modifier(&mut self, id: ModifierID, value: f32) {
        if !self.stack.contains_key(&id) {
            self.stack.insert(id, Default::default());
        }

        self.stack
            .get_mut(&id)
            .unwrap()
            .modify_multiplicative(value);
    }

    pub fn add_additive_modifier(&mut self, id: ModifierID, value: f32) {
        if !self.stack.contains_key(&id) {
            self.stack.insert(id, Default::default());
        }

        self.stack
            .get_mut(&id)
            .unwrap()
            .modify_additive(value);
    }
}


pub fn apply_modifier_if_present(query_result: Option<&ModifierStack>, id: ModifierID, base_value: f32) -> f32 {
    let modifier = query_result.and_then(|stack| stack.get(id)).unwrap_or_default();

    modifier.apply_to_base_value(base_value)
}

pub const DAMAGE_MODIFIER: ModifierID = ModifierID("basic_damage");

pub const PLAYER_SPEED_MODIFIER: ModifierID = ModifierID("player_speed");
pub const PROJECTILE_COUNT_MODIFIER: ModifierID = ModifierID("projectile_count");
pub const PROJECTILE_DURATION_MODIFIER: ModifierID = ModifierID("projectile_duration");