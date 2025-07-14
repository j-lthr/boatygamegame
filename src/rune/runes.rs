use bevy::prelude::*;

use crate::{
    modifiers::*,
    rune::{ModifierRune},
};



pub const SPEED_RUNE: ModifierRune = ModifierRune {
    modifier_id: PLAYER_SPEED_MODIFIER,
    multiplicative_factor: 1.0,
    additive_factor: 0.1,
    color: Color::linear_rgb(0.5, 1.0, 100.0),
};

pub const MULTISHOT_RUNE: ModifierRune = ModifierRune {
    modifier_id: PROJECTILE_COUNT_MODIFIER,
    multiplicative_factor: 1.0,
    additive_factor: 1.0,
    color: Color::linear_rgb(100.0, 0.0, 0.0),
};

pub const DAMAGE_RUNE: ModifierRune = ModifierRune {
    modifier_id: DAMAGE_MODIFIER,
    multiplicative_factor: 1.0,
    additive_factor: 0.1,
    color: Color::linear_rgb(100.0, 50.0, 0.0),
};
