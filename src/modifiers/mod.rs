use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Copy, Clone)]
pub struct CompoundModifier {
    multiplicative_factor: f32,
    additive_factor: f32,
}

impl Default for CompoundModifier {
    fn default() -> Self {
        Self {
            multiplicative_factor: 1.0,
            additive_factor: 0.0,
        }
    }
}

impl CompoundModifier {
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
pub enum ModifierType {
    Additive(f32),
    Multiplicative(f32),
}

#[derive(Copy, Clone, Debug)]
pub struct Modifier {
    pub id: ModifierID,
    pub typ: ModifierType,
}

impl Modifier {
    pub const fn additive(id: ModifierID, value: f32) -> Self {
        Self {
            id,
            typ: ModifierType::Additive(value),
        }
    }

    pub const fn multiplicative(id: ModifierID, value: f32) -> Self {
        Self {
            id,
            typ: ModifierType::Multiplicative(value),
        }
    }
}

#[derive(Component, Default)]
pub struct ModifierStack {
    stack: HashMap<ModifierID, CompoundModifier>,
}

impl ModifierStack {
  
    pub fn get(&self, id: ModifierID) -> Option<CompoundModifier> {
        self.stack.get(&id).cloned()
    }

    pub fn add_modifier(&mut self, stat: Modifier) {
        if !self.stack.contains_key(&stat.id) {
            self.stack.insert(stat.id, Default::default());
        }

        let modifier = self.stack.get_mut(&stat.id).unwrap();
        match stat.typ {
            ModifierType::Additive(value) => modifier.modify_additive(value),
            ModifierType::Multiplicative(value) => modifier.modify_multiplicative(value),
        }
    }

    pub fn add_multiplicative_modifier(&mut self, id: ModifierID, value: f32) {
        self.add_modifier(Modifier { id, typ: ModifierType::Multiplicative(value) });
    }

    pub fn add_additive_modifier(&mut self, id: ModifierID, value: f32) {
        self.add_modifier(Modifier { id, typ: ModifierType::Additive(value) });
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