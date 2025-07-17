use bevy::prelude::*;

use crate::{
    modifiers::*,
    rune::{ModifierRune},
};


pub const SPEED_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::additive(PLAYER_SPEED_MODIFIER, 0.1),
    color: Color::linear_rgb(0.5, 1.0, 100.0),
};

pub const MULTISHOT_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::flat_added(PROJECTILE_COUNT_MODIFIER, 1.0),
    color: Color::linear_rgb(100.0, 0.0, 0.0),
};

pub const DAMAGE_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::additive(DAMAGE_MODIFIER, 0.1),
    color: Color::linear_rgb(100.0, 50.0, 0.0),
};

pub const AOE_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::additive(AOE_RADIUS_MODIFIER, 0.1),
    color: Color::linear_rgb(0.0, 50.0, 50.0),
};

pub const MAX_HEALTH_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::additive(MAX_HEALTH_MODIFIER, 0.1),
    color: Color::linear_rgb(0.0, 50.0, 0.0),
};

pub const HEALTH_REGEN_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::flat_added(HEALTH_REGEN_MODIFIER, 5.0),
    color: Color::linear_rgb(100.0, 50.0, 100.0),
};

pub const COOLDOWN_RECOVERY_RATE_RUNE: ModifierRune = ModifierRune {
    modifier: Modifier::additive(COOLDOWN_RECOVERY_RATE_MODIFIER, 0.1),
    color: Color::linear_rgb(100.0, 50.0, 100.0),
};